package com.audiopc.audiopc

import android.app.Activity
import android.content.Context
import android.media.MediaMetadataRetriever
import android.os.Handler
import androidx.annotation.OptIn
import androidx.media3.common.util.UnstableApi
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.plugin.common.EventChannel
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import io.flutter.plugin.common.MethodChannel.MethodCallHandler
import io.flutter.plugin.common.MethodChannel.Result

@OptIn(UnstableApi::class)
/** AudiopcPlugin */
class AudiopcPlugin : FlutterPlugin, MethodCallHandler {
    /// The MethodChannel that will the communication between Flutter and native Android
    ///
    /// This local reference serves to register the plugin with the Flutter Engine and unregister it
    /// when the Flutter Engine is detached from the Activity
    private lateinit var channel: MethodChannel
    private lateinit var eventChannel: EventChannel
    private lateinit var eventStreamHandler: EventStreamHandler
    private lateinit var context: Context
    private lateinit var handler: Handler

    private val audioPlayers: MutableMap<String, AudioPlayer> = mutableMapOf()

    override fun onAttachedToEngine(flutterPluginBinding: FlutterPlugin.FlutterPluginBinding) {
        channel = MethodChannel(flutterPluginBinding.binaryMessenger, "audiopc/methodChannel")
        channel.setMethodCallHandler(this)
        eventChannel = EventChannel(flutterPluginBinding.binaryMessenger, "audiopc/eventChannel")
        eventStreamHandler = EventStreamHandler()
        eventChannel.setStreamHandler(eventStreamHandler)
        context = flutterPluginBinding.applicationContext
        handler = Handler(context.mainLooper)
    }

    override fun onMethodCall(call: MethodCall, result: Result) {
        val id = call.argument<String>("id")
        if (call.method == "init") {
            if (id != null) {
                val audioPlayer = AudioPlayer(id, eventStreamHandler.sink!!, context, handler)
                audioPlayers[id] = audioPlayer
            } else {
                result.error(
                    "NO_ID_PROVIDED",
                    "ID not found",
                    "Please provide a valid ID"
                )
            }
        } else {
            val audioPlayer = audioPlayers[id]
            when (call.method) {
                "setVolume" -> {
                    audioPlayer?.setVolume(call.argument<Double>("volume") ?: 1.0)
                }

                "setSource" -> {
                    val path = call.argument<String>("path")
                    if (path != null) {
                        audioPlayer?.setSource(path)
                    }
                }

                "play" -> {
                    audioPlayer?.play()
                }

                "pause" -> {
                    audioPlayer?.pause()
                }
                "seek" -> {
                    audioPlayer?.seek(call.argument<Double>("position") ?: 0.0)
                }

                "getPosition" -> {
                    result.success(audioPlayer?.getPosition() ?: 0.0)
                }

                "setRate" -> {
                    audioPlayer?.setRate(call.argument<Double>("rate") ?: 1.0)
                }

                "close" -> {
                    audioPlayer?.shutdown()
                    audioPlayers.remove(id)
                }

                "getMetaData" -> {
                    val path = call.argument<String>("path")
                    if (path != null) {
                        val mediaMetadataRetriever = MediaMetadataRetriever()
                        mediaMetadataRetriever.setDataSource(path)
                        val metadataMap = mutableMapOf<String, Any?>(
                            "title" to mediaMetadataRetriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_TITLE),
                            "artist" to mediaMetadataRetriever.extractMetadata(
                                MediaMetadataRetriever.METADATA_KEY_ARTIST
                            ),
                            "album" to mediaMetadataRetriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_ALBUM),
                            "artwork" to mediaMetadataRetriever.embeddedPicture,
                            "timeReleased" to mediaMetadataRetriever.extractMetadata(
                                MediaMetadataRetriever.METADATA_KEY_YEAR
                            ),
                            "genre" to mediaMetadataRetriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_GENRE),
                        )
                        result.success(metadataMap)
                    }
                }

                else -> {
                    result.notImplemented()
                }
            }
        }
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        channel.setMethodCallHandler(null)
        eventChannel.setStreamHandler(null)
    }
}
