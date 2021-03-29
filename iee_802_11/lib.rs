use anyhow::Result;

pub mod frame_control;
pub mod frame_types;
pub mod payload;

use crate::frame_control::FrameControl;
use crate::payload::*;

/// This represents a full IEE 800.11 frame.
/// It's devided into the Header,
pub struct Frame {
    pub control: FrameControl,
    pub payload: Payload,
    //frame_check_sequence: [u8; 4],
}

impl Frame {
    pub fn parse(input: &[u8]) -> Result<Frame> {
        let frame_control = FrameControl::parse(&input[0..2]);
        println!(
            "Type/Subtype: {:?}, {:?}",
            frame_control.frame_type, frame_control.frame_subtype
        );
        println!("Payload bytes: {:?}", &input);

        let payload = Payload::parse(&frame_control, &input);

        Ok(Frame {
            control: frame_control,
            payload,
        })
    }
}
