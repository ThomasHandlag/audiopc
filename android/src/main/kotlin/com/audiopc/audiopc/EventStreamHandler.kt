package com.audiopc.audiopc

import io.flutter.plugin.common.EventChannel
import io.flutter.plugin.common.EventChannel.StreamHandler

class EventStreamHandler : StreamHandler {
    var sink: EventChannel.EventSink? = null
    override fun onListen(arguments: Any?, events: EventChannel.EventSink?) {
        if (events != null) {
            sink = events
        }
    }

    override fun onCancel(arguments: Any?) {
        sink = null
    }
}