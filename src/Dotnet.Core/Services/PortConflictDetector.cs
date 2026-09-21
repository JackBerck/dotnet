using System;
using System.Diagnostics;
using System.Linq;
using System.Net.NetworkInformation;
using System.Runtime.InteropServices;

namespace dotnet.Services;

public class PortConflictInfo
{
    public int Port { get; set; }
    public bool IsInUse { get; set; }
    public int ProcessId { get; set; }
    public string ProcessName { get; set; } = string.Empty;
}

public class PortConflictDetector
{
    public enum TCP_TABLE_CLASS
    {
        TCP_TABLE_BASIC_LISTENER,
        TCP_TABLE_BASIC_CONNECTIONS,
        TCP_TABLE_BASIC_ALL,
        TCP_TABLE_OWNER_PID_LISTENER,
        TCP_TABLE_OWNER_PID_CONNECTIONS,
        TCP_TABLE_OWNER_PID_ALL,
        TCP_TABLE_OWNER_MODULE_LISTENER,
        TCP_TABLE_OWNER_MODULE_CONNECTIONS,
        TCP_TABLE_OWNER_MODULE_ALL
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct MIB_TCPROW_OWNER_PID
    {
        public uint state;
        public uint localAddr;
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 4)]
        public byte[] localPort;
        public uint remoteAddr;
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 4)]
        public byte[] remotePort;
        public uint owningPid;
    }

    [DllImport("iphlpapi.dll", SetLastError = true)]
    static extern uint GetExtendedTcpTable(IntPtr pTcpTable, ref int dwOutBufLen, bool sort, int ipVersion, TCP_TABLE_CLASS tblClass, uint reserved);

    public static int? GetProcessIdByPort(int port)
    {
        int buffSize = 0;
        uint ret = GetExtendedTcpTable(IntPtr.Zero, ref buffSize, true, 2 /* AF_INET */, TCP_TABLE_CLASS.TCP_TABLE_OWNER_PID_LISTENER, 0);
        
        if (ret != 0 && ret != 122 /* ERROR_INSUFFICIENT_BUFFER */)
        {
            return null;
        }

        IntPtr buffTable = Marshal.AllocHGlobal(buffSize);
        try
        {
            ret = GetExtendedTcpTable(buffTable, ref buffSize, true, 2, TCP_TABLE_CLASS.TCP_TABLE_OWNER_PID_LISTENER, 0);
            if (ret != 0)
            {
                return null;
            }

            int numEntries = Marshal.ReadInt32(buffTable);
            IntPtr rowPtr = new IntPtr(buffTable.ToInt64() + 4);
            
            for (int i = 0; i < numEntries; i++)
            {
                var row = Marshal.PtrToStructure<MIB_TCPROW_OWNER_PID>(rowPtr);
                ushort rowPort = (ushort)((row.localPort[0] << 8) | row.localPort[1]);
                
                if (rowPort == port)
                {
                    return (int)row.owningPid;
                }
                
                rowPtr = new IntPtr(rowPtr.ToInt64() + Marshal.SizeOf(typeof(MIB_TCPROW_OWNER_PID)));
            }
        }
        finally
        {
            Marshal.FreeHGlobal(buffTable);
        }

        return null;
    }

    public static PortConflictInfo CheckPort(int port)
    {
        int? pid = GetProcessIdByPort(port);
        if (pid.HasValue)
        {
            string name = "Unknown Process";
            try
            {
                var proc = Process.GetProcessById(pid.Value);
                name = proc.ProcessName;
            }
            catch
            {
                // Process might have ended or permission restricted
            }

            return new PortConflictInfo
            {
                Port = port,
                IsInUse = true,
                ProcessId = pid.Value,
                ProcessName = name
            };
        }

        return new PortConflictInfo
        {
            Port = port,
            IsInUse = false,
            ProcessId = 0,
            ProcessName = "None"
        };
    }
}
