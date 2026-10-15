//! Service Orchestrator: dependency-ordered startup and reverse-ordered graceful shutdown.

use crate::models::DevServiceDescriptor;
use crate::services::profile_store::ServiceProfile;
use std::future::Future;

pub struct ServiceOrchestrator;

impl ServiceOrchestrator {
    /// Get the startup priority score for a service (lower starts first).
    pub fn get_start_priority(service_id: &str) -> u32 {
        match service_id.to_lowercase().as_str() {
            "mysql" => 1,
            "postgres" | "postgresql" => 2,
            "redis" | "memurai" => 3,
            "php-cgi" | "php" => 4,
            "nginx" => 5,
            _ => 10,
        }
    }

    /// Sort services for startup (databases first, web servers last).
    pub fn order_for_startup<'a>(
        services: &'a [DevServiceDescriptor],
        profile: &ServiceProfile,
    ) -> Vec<&'a DevServiceDescriptor> {
        let mut filtered: Vec<&'a DevServiceDescriptor> = services
            .iter()
            .filter(|s| {
                profile
                    .service_ids
                    .iter()
                    .any(|id| id.eq_ignore_ascii_case(&s.id))
            })
            .collect();

        filtered.sort_by_key(|s| Self::get_start_priority(&s.id));
        filtered
    }

    /// Sort services for shutdown (web servers first, databases last).
    pub fn order_for_shutdown<'a>(
        services: &'a [DevServiceDescriptor],
        profile: &ServiceProfile,
    ) -> Vec<&'a DevServiceDescriptor> {
        let mut ordered = Self::order_for_startup(services, profile);
        ordered.reverse();
        ordered
    }

    /// Asynchronously start profile services in dependency order.
    pub async fn start_profile_services<F, Fut>(
        services: &[DevServiceDescriptor],
        profile: &ServiceProfile,
        mut start_fn: F,
    ) where
        F: FnMut(&DevServiceDescriptor) -> Fut,
        Fut: Future<Output = bool>,
    {
        let ordered = Self::order_for_startup(services, profile);
        tracing::info!(
            "Orchestrator: Starting {} services for profile '{}'",
            ordered.len(),
            profile.name
        );

        for svc in ordered {
            let ok = start_fn(svc).await;
            if !ok {
                tracing::warn!("Orchestrator: Failed starting service '{}'", svc.id);
            }
        }
    }

    /// Asynchronously stop profile services in reverse dependency order.
    pub async fn stop_profile_services<F, Fut>(
        services: &[DevServiceDescriptor],
        profile: &ServiceProfile,
        mut stop_fn: F,
    ) where
        F: FnMut(&DevServiceDescriptor) -> Fut,
        Fut: Future<Output = bool>,
    {
        let ordered = Self::order_for_shutdown(services, profile);
        tracing::info!(
            "Orchestrator: Stopping {} services for profile '{}'",
            ordered.len(),
            profile.name
        );

        for svc in ordered {
            let ok = stop_fn(svc).await;
            if !ok {
                tracing::warn!("Orchestrator: Failed stopping service '{}'", svc.id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ServiceType;

    fn make_test_service(id: &str) -> DevServiceDescriptor {
        DevServiceDescriptor {
            id: id.to_string(),
            name: id.to_string(),
            description: String::new(),
            port: 80,
            service_type: ServiceType::ManagedProcess,
            windows_service_name: None,
            binary_name: None,
            default_args: vec![],
            auto_start: true,
        }
    }

    #[test]
    fn test_orchestration_ordering() {
        let all_services = vec![
            make_test_service("nginx"),
            make_test_service("mysql"),
            make_test_service("php-cgi"),
            make_test_service("redis"),
        ];

        let profile = ServiceProfile {
            id: "standalone".into(),
            name: "Standalone".into(),
            service_ids: vec![
                "nginx".into(),
                "mysql".into(),
                "php-cgi".into(),
                "redis".into(),
            ],
            optional_service_ids: vec![],
        };

        let start_order = ServiceOrchestrator::order_for_startup(&all_services, &profile);
        let start_ids: Vec<&str> = start_order.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(start_ids, vec!["mysql", "redis", "php-cgi", "nginx"]);

        let stop_order = ServiceOrchestrator::order_for_shutdown(&all_services, &profile);
        let stop_ids: Vec<&str> = stop_order.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(stop_ids, vec!["nginx", "php-cgi", "redis", "mysql"]);
    }
}
