microtype! {
    #[derive(Debug, Clone, PartialEq)]
    #[int]
    pub i64 {
        CorrelationId,
        ControlSessionId,
        SubscriptionId,
    }

    #[derive(Debug, Clone, PartialEq)]
    #[int]
    pub i32 {
        StreamId,
        SessionId
    }

    #[derive(Debug, Clone)]
    #[string]
    pub String {
        #[derive(PartialEq)]
        ChannelUriStr
    }
}

pub struct ConnectionInfo {
    pub channel_uri: ChannelUriStr,
    pub stream_id: StreamId,
    pub session_id: SessionId,
    pub correlation_id: CorrelationId,
}
