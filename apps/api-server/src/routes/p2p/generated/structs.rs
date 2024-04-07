// Automatically generated rust module for 'structs.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct VideoInfo {
    pub hash: Option<String>,
    pub file_name: Option<String>,
    pub metadata: Option<structs::Metadata>,
    pub video_frame: Vec<structs::VideoFrame>,
    pub video_clip: Vec<structs::VideoClip>,
    pub video_frame_caption: Vec<structs::VideoFrameCaption>,
    pub video_transcript: Vec<structs::VideoTranscript>,
}

impl<'a> MessageRead<'a> for VideoInfo {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.hash = Some(r.read_string(bytes)?.to_owned()),
                Ok(18) => msg.file_name = Some(r.read_string(bytes)?.to_owned()),
                Ok(26) => msg.metadata = Some(r.read_message::<structs::Metadata>(bytes)?),
                Ok(34) => msg.video_frame.push(r.read_message::<structs::VideoFrame>(bytes)?),
                Ok(42) => msg.video_clip.push(r.read_message::<structs::VideoClip>(bytes)?),
                Ok(50) => msg.video_frame_caption.push(r.read_message::<structs::VideoFrameCaption>(bytes)?),
                Ok(58) => msg.video_transcript.push(r.read_message::<structs::VideoTranscript>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for VideoInfo {
    fn get_size(&self) -> usize {
        0
        + self.hash.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.file_name.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.metadata.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.video_frame.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        + self.video_clip.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        + self.video_frame_caption.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        + self.video_transcript.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.hash { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.file_name { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.metadata { w.write_with_tag(26, |w| w.write_message(s))?; }
        for s in &self.video_frame { w.write_with_tag(34, |w| w.write_message(s))?; }
        for s in &self.video_clip { w.write_with_tag(42, |w| w.write_message(s))?; }
        for s in &self.video_frame_caption { w.write_with_tag(50, |w| w.write_message(s))?; }
        for s in &self.video_transcript { w.write_with_tag(58, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Metadata {
    pub id: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<i32>,
    pub bit_rate: Option<i32>,
    pub size: Option<i32>,
    pub mime_type: Option<String>,
    pub has_audio: Option<bool>,
    pub description: Option<String>,
    pub asset_object_id: Option<i32>,
}

impl<'a> MessageRead<'a> for Metadata {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.id = Some(r.read_int32(bytes)?),
                Ok(16) => msg.width = Some(r.read_int32(bytes)?),
                Ok(24) => msg.height = Some(r.read_int32(bytes)?),
                Ok(32) => msg.duration = Some(r.read_int32(bytes)?),
                Ok(40) => msg.bit_rate = Some(r.read_int32(bytes)?),
                Ok(48) => msg.size = Some(r.read_int32(bytes)?),
                Ok(58) => msg.mime_type = Some(r.read_string(bytes)?.to_owned()),
                Ok(64) => msg.has_audio = Some(r.read_bool(bytes)?),
                Ok(74) => msg.description = Some(r.read_string(bytes)?.to_owned()),
                Ok(80) => msg.asset_object_id = Some(r.read_int32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Metadata {
    fn get_size(&self) -> usize {
        0
        + self.id.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.width.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.height.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.duration.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.bit_rate.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.size.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.mime_type.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.has_audio.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.description.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.asset_object_id.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.id { w.write_with_tag(8, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.width { w.write_with_tag(16, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.height { w.write_with_tag(24, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.duration { w.write_with_tag(32, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.bit_rate { w.write_with_tag(40, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.size { w.write_with_tag(48, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.mime_type { w.write_with_tag(58, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.has_audio { w.write_with_tag(64, |w| w.write_bool(*s))?; }
        if let Some(ref s) = self.description { w.write_with_tag(74, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.asset_object_id { w.write_with_tag(80, |w| w.write_int32(*s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct VideoFrame {
    pub id: Option<i32>,
    pub file_identifier: Option<String>,
    pub timestamp: Option<i32>,
    pub video_clip_id: Option<i32>,
}

impl<'a> MessageRead<'a> for VideoFrame {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.id = Some(r.read_int32(bytes)?),
                Ok(18) => msg.file_identifier = Some(r.read_string(bytes)?.to_owned()),
                Ok(24) => msg.timestamp = Some(r.read_int32(bytes)?),
                Ok(32) => msg.video_clip_id = Some(r.read_int32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for VideoFrame {
    fn get_size(&self) -> usize {
        0
        + self.id.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.file_identifier.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.timestamp.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.video_clip_id.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.id { w.write_with_tag(8, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.file_identifier { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.timestamp { w.write_with_tag(24, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.video_clip_id { w.write_with_tag(32, |w| w.write_int32(*s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct VideoClip {
    pub id: Option<i32>,
    pub file_identifier: Option<String>,
    pub start_timestamp: Option<i32>,
    pub end_timestamp: Option<i32>,
    pub caption: Option<String>,
}

impl<'a> MessageRead<'a> for VideoClip {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.id = Some(r.read_int32(bytes)?),
                Ok(18) => msg.file_identifier = Some(r.read_string(bytes)?.to_owned()),
                Ok(24) => msg.start_timestamp = Some(r.read_int32(bytes)?),
                Ok(32) => msg.end_timestamp = Some(r.read_int32(bytes)?),
                Ok(42) => msg.caption = Some(r.read_string(bytes)?.to_owned()),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for VideoClip {
    fn get_size(&self) -> usize {
        0
        + self.id.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.file_identifier.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.start_timestamp.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.end_timestamp.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.caption.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.id { w.write_with_tag(8, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.file_identifier { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.start_timestamp { w.write_with_tag(24, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.end_timestamp { w.write_with_tag(32, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.caption { w.write_with_tag(42, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct VideoFrameCaption {
    pub id: Option<i32>,
    pub caption: Option<String>,
    pub video_frame_id: Option<i32>,
}

impl<'a> MessageRead<'a> for VideoFrameCaption {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.id = Some(r.read_int32(bytes)?),
                Ok(18) => msg.caption = Some(r.read_string(bytes)?.to_owned()),
                Ok(24) => msg.video_frame_id = Some(r.read_int32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for VideoFrameCaption {
    fn get_size(&self) -> usize {
        0
        + self.id.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.caption.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.video_frame_id.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.id { w.write_with_tag(8, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.caption { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.video_frame_id { w.write_with_tag(24, |w| w.write_int32(*s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct VideoTranscript {
    pub id: Option<i32>,
    pub file_identifier: Option<String>,
    pub start_timestamp: Option<i32>,
    pub end_timestamp: Option<i32>,
    pub text: Option<String>,
}

impl<'a> MessageRead<'a> for VideoTranscript {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.id = Some(r.read_int32(bytes)?),
                Ok(18) => msg.file_identifier = Some(r.read_string(bytes)?.to_owned()),
                Ok(24) => msg.start_timestamp = Some(r.read_int32(bytes)?),
                Ok(32) => msg.end_timestamp = Some(r.read_int32(bytes)?),
                Ok(42) => msg.text = Some(r.read_string(bytes)?.to_owned()),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for VideoTranscript {
    fn get_size(&self) -> usize {
        0
        + self.id.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.file_identifier.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.start_timestamp.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.end_timestamp.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.text.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.id { w.write_with_tag(8, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.file_identifier { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.start_timestamp { w.write_with_tag(24, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.end_timestamp { w.write_with_tag(32, |w| w.write_int32(*s))?; }
        if let Some(ref s) = self.text { w.write_with_tag(42, |w| w.write_string(&**s))?; }
        Ok(())
    }
}

