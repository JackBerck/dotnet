//! Native Windows IP Helper API (iphlpapi.dll) wrappers for high-speed port inspection.

use crate::error::{Result, WindowsError};
use std::net::{Ipv4Addr, SocketAddrV4};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCPROW_OWNER_PID, MIB_TCPTABLE_OWNER_PID, TCP_TABLE_OWNER_PID_LISTENER,
};
use windows_sys::Win32::Networking::WinSock::AF_INET;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcpListenerEntry {
    pub local_addr: SocketAddrV4,
    pub port: u16,
    pub owning_pid: u32,
}

/// Retrieve all active IPv4 TCP listeners with their owning Process IDs.
/// Uses native Win32 `GetExtendedTcpTable` without spawning child processes.
pub fn get_active_tcp_listeners() -> Result<Vec<TcpListenerEntry>> {
    let mut size: u32 = 0;

    // First call to determine required buffer size
    unsafe {
        GetExtendedTcpTable(
            std::ptr::null_mut(),
            &mut size,
            1, // sort = true
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_LISTENER,
            0,
        );
    }

    if size == 0 {
        return Ok(Vec::new());
    }

    // Allocate buffer
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

    let mut entries = Vec::new();

    unsafe {
        let table = buffer.as_ptr() as *const MIB_TCPTABLE_OWNER_PID;
        let num_entries = (*table).dwNumEntries as usize;
        let row_ptr = std::ptr::addr_of!((*table).table) as *const MIB_TCPROW_OWNER_PID;

        for i in 0..num_entries {
            let row = *row_ptr.add(i);

            // dwLocalPort is stored in network byte order (big endian) in the upper 16 bits or first two bytes
            let raw_port = row.dwLocalPort as u16;
            let port = u16::from_be(raw_port);

            let ip_bytes = row.dwLocalAddr.to_ne_bytes();
            let ip = Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]);

            entries.push(TcpListenerEntry {
                local_addr: SocketAddrV4::new(ip, port),
                port,
                owning_pid: row.dwOwningPid,
            });
        }
    }

    Ok(entries)
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
