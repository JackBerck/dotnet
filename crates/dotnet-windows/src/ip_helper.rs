//! Native Windows IP Helper API (iphlpapi.dll) wrappers for high-speed port inspection.
//! Supports both IPv4 and IPv6 listeners without spawning child processes.

use crate::error::{Result, WindowsError};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCP6TABLE_OWNER_PID, MIB_TCPROW_OWNER_PID,
    MIB_TCPTABLE_OWNER_PID, TCP_TABLE_OWNER_PID_LISTENER,
};
use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcpListenerEntry {
    pub local_addr: SocketAddr,
    pub port: u16,
    pub owning_pid: u32,
}

/// Retrieve all active TCP listeners (IPv4 and IPv6) with their owning Process IDs.
pub fn get_active_tcp_listeners() -> Result<Vec<TcpListenerEntry>> {
    let mut entries = Vec::new();

    // 1. Query IPv4 listeners
    get_ipv4_listeners(&mut entries)?;

    // 2. Query IPv6 listeners
    get_ipv6_listeners(&mut entries)?;

    Ok(entries)
}

fn get_ipv4_listeners(entries: &mut Vec<TcpListenerEntry>) -> Result<()> {
    let mut size: u32 = 0;

    unsafe {
        GetExtendedTcpTable(
            std::ptr::null_mut(),
            &mut size,
            1,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_LISTENER,
            0,
        );
    }

    if size == 0 {
        return Ok(());
    }

    let mut buffer: Vec<u8> = vec![0u8; size as usize];
    let result = unsafe {
        GetExtendedTcpTable(
            buffer.as_mut_ptr() as *mut _,
            &mut size,
            1,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_LISTENER,
            0,
        )
    };

    if result != 0 {
        return Err(WindowsError::IpHelperError(result));
    }

    unsafe {
        let table = buffer.as_ptr() as *const MIB_TCPTABLE_OWNER_PID;
        let num_entries = (*table).dwNumEntries as usize;
        let row_ptr = std::ptr::addr_of!((*table).table) as *const MIB_TCPROW_OWNER_PID;

        for i in 0..num_entries {
            let row = *row_ptr.add(i);
            let port = u16::from_be(row.dwLocalPort as u16);
            let ip_bytes = row.dwLocalAddr.to_ne_bytes();
            let ip = Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]);

            entries.push(TcpListenerEntry {
                local_addr: SocketAddr::new(IpAddr::V4(ip), port),
                port,
                owning_pid: row.dwOwningPid,
            });
        }
    }

    Ok(())
}

fn get_ipv6_listeners(entries: &mut Vec<TcpListenerEntry>) -> Result<()> {
    let mut size: u32 = 0;

    unsafe {
        GetExtendedTcpTable(
            std::ptr::null_mut(),
            &mut size,
            1,
            AF_INET6 as u32,
            TCP_TABLE_OWNER_PID_LISTENER,
            0,
        );
    }

    if size == 0 {
        return Ok(());
    }

    let mut buffer: Vec<u8> = vec![0u8; size as usize];
    let result = unsafe {
        GetExtendedTcpTable(
            buffer.as_mut_ptr() as *mut _,
            &mut size,
            1,
            AF_INET6 as u32,
            TCP_TABLE_OWNER_PID_LISTENER,
            0,
        )
    };

    if result != 0 {
        // Not all systems or environments support or have active IPv6, treat non-zero gracefully if not fatal
        tracing::debug!("IPv6 GetExtendedTcpTable returned non-zero status: {result}");
        return Ok(());
    }

    unsafe {
        let table = buffer.as_ptr() as *const MIB_TCP6TABLE_OWNER_PID;
        let num_entries = (*table).dwNumEntries as usize;
        let row_ptr = std::ptr::addr_of!((*table).table) as *const MIB_TCP6ROW_OWNER_PID;

        for i in 0..num_entries {
            let row = *row_ptr.add(i);
            let port = u16::from_be(row.dwLocalPort as u16);
            let ip = Ipv6Addr::from(row.ucLocalAddr);

            entries.push(TcpListenerEntry {
                local_addr: SocketAddr::new(IpAddr::V6(ip), port),
                port,
                owning_pid: row.dwOwningPid,
            });
        }
    }

    Ok(())
}

/// Check if a specific TCP port is currently bound, and return owning PID if so.
pub fn get_owner_pid_for_port(port: u16) -> Result<Option<u32>> {
    let listeners = get_active_tcp_listeners()?;
    for entry in listeners {
        if entry.port == port {
            return Ok(Some(entry.owning_pid));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_active_listeners_does_not_fail() {
        let listeners = get_active_tcp_listeners().expect("Must query TCP listeners without error");
        assert!(
            !listeners.is_empty(),
            "Windows system should have at least some active listening ports"
        );
    }
}
