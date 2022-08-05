// States
pub const ZONE_STATE_CONFIGURED: i32 = 0; //(never see)
pub const ZONE_STATE_INCOMPLETE: i32 = 1; //(never see)
pub const ZONE_STATE_INSTALLED: i32 = 2;
pub const ZONE_STATE_READY: i32 = 3;
pub const ZONE_STATE_RUNNING: i32 = 4;
pub const ZONE_STATE_SHUTTING_DOWN: i32 = 5;
pub const ZONE_STATE_DOWN: i32 = 6;
pub const ZONE_STATE_MOUNTED: i32 = 7;

// cmd
pub const ZONE_CMD_READY: i32 = 0;
pub const ZONE_CMD_BOOT: i32 = 1;
pub const ZONE_CMD_FORCEBOOT: i32 = 2;
pub const ZONE_CMD_REBOOT: i32 = 3;
pub const ZONE_CMD_HALT: i32 = 4;
pub const ZONE_CMD_UNINSTALLING: i32 = 5;
pub const ZONE_CMD_MOUNT: i32 = 6;
pub const ZONE_CMD_FORCEMOUNT: i32 = 7;
pub const ZONE_CMD_UNMOUNT: i32 = 8;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ZoneSubProcExitCode(i32);

impl ZoneSubProcExitCode {
    pub const ZONE_SUBPROC_OK: ZoneSubProcExitCode = ZoneSubProcExitCode(0);
    pub const ZONE_SUBPROC_USAGE: ZoneSubProcExitCode = ZoneSubProcExitCode(253);
    pub const ZONE_SUBPROC_NOTCOMPLETE: ZoneSubProcExitCode = ZoneSubProcExitCode(254);
    pub const ZONE_SUBPROC_FATAL: ZoneSubProcExitCode = ZoneSubProcExitCode(255);
}

