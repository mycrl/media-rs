use std::collections::HashMap;

use rml_rtmp::messages::*;
use rml_rtmp::rml_amf0::{Amf0Value, Amf0Value::*};

pub enum Msg {
    WindowAcknowledgement,
    SetPeerBandwidth,
    ConnectSuccess,
    PublishSuccess,
    CreateSreamSuccess,
}

impl Msg {
    fn window_acknowledgement() -> RtmpMessage {
        RtmpMessage::WindowAcknowledgement { size: 5000000 }
    }

    fn set_peer_bandwidth() -> RtmpMessage {
        RtmpMessage::SetPeerBandwidth {
            limit_type: PeerBandwidthLimitType::Hard,
            size: 5000000,
        }
    }

    fn connect_success() -> RtmpMessage {
        RtmpMessage::Amf0Command {
            additional_arguments: vec![CommandArgs::ConnectSuccess.into()],
            command_object: CommandObj::ConnectSuccess.into(),
            command_name: "_result".to_string(),
            transaction_id: 1.0,
        }
    }

    fn publish_success() -> RtmpMessage {
        RtmpMessage::Amf0Command {
            additional_arguments: vec![CommandArgs::PublishSuccess.into()],
            command_name: "onStatus".to_string(),
            command_object: Null,
            transaction_id: 0.0,
        }
    }

    fn create_sream_success() -> RtmpMessage {
        RtmpMessage::Amf0Command {
            additional_arguments: vec![Number(1.0)],
            command_name: "_result".to_string(),
            command_object: Null,
            transaction_id: 4.0,
        }
    }
}

impl From<Msg> for RtmpMessage {
    fn from(val: Msg) -> Self {
        match val {
            Msg::WindowAcknowledgement => Msg::window_acknowledgement(),
            Msg::SetPeerBandwidth => Msg::set_peer_bandwidth(),
            Msg::ConnectSuccess => Msg::connect_success(),
            Msg::PublishSuccess => Msg::publish_success(),
            Msg::CreateSreamSuccess => Msg::create_sream_success(),
        }
    }
}

pub enum CommandArgs {
    ConnectSuccess,
    PublishSuccess,
}

impl CommandArgs {
    #[rustfmt::skip]
    fn connect_success() -> HashMap<String, Amf0Value> {
        let mut args = HashMap::new();
        args.insert("level".to_string(), Utf8String("status".to_string()));
        args.insert("code".to_string(), Utf8String("NetConnection.Connect.Success".to_string()));
        args.insert("description".to_string(), Utf8String("Connection succeeded.".to_string()));
        args.insert("objectEncoding".to_string(), Number(0.0));
        args
    }

    #[rustfmt::skip]
    fn publish_success() -> HashMap<String, Amf0Value> {
        let mut args = HashMap::new();
        args.insert("level".to_string(), Utf8String("status".to_string()));
        args.insert("code".to_string(), Utf8String("NetStream.Publish.Start".to_string()));
        args.insert("description".to_string(), Utf8String("Start publishing".to_string()));
        args
    }
}

impl From<CommandArgs> for Amf0Value {
    fn from(val: CommandArgs) -> Self {
        Object(match val {
            CommandArgs::ConnectSuccess => CommandArgs::connect_success(),
            CommandArgs::PublishSuccess => CommandArgs::publish_success(),
        })
    }
}

pub enum CommandObj {
    ConnectSuccess,
}

impl CommandObj {
    #[rustfmt::skip]
    fn connect_success() -> HashMap<String, Amf0Value> {
        let mut obj = HashMap::new();
        obj.insert("fmsVer".to_string(), Utf8String("FMS/3,0,1,123".to_string()));
        obj.insert("capabilities".to_string(), Number(31.0));
        obj
    }
}

impl From<CommandObj> for Amf0Value {
    fn from(val: CommandObj) -> Self {
        Object(match val {
            CommandObj::ConnectSuccess => CommandObj::connect_success(),
        })
    }
}
