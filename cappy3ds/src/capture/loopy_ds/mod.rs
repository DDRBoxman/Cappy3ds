USBDS_VID               = 0x16D0,
USBDS_PID               = 0x0647,

    //device vendor requests:
    CMDIN_STATUS            = 0x31,
        // Returns device status:
        // struct {
        //     u32 framecount,              //free running frame counter
        //     u8 lcd_on,
        //     u8 capture_in_progress,
        // }
    CMDIN_FRAMEINFO         = 0x30,
        // Returns record of last captured frame:
        // struct {
        //     u8 bitmap[48],     //bitmap of lines sent (1 bit per half-line)
        //     u32 frame,         //frame number
        //     u8 valid,          //0 if capture timed out (LCD is inactive)
        // }
    CMDOUT_CAPTURE_START    = 0x30,
        // capture new frame
    CMDOUT_CAPTURE_STOP     = 0x31,
        // stop capture in progress and reset frame counter to 0