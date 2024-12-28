package com.audiopc.audiopc

import android.content.Context
import android.os.Handler
import androidx.annotation.OptIn
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.DefaultRenderersFactory
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.Renderer
import androidx.media3.exoplayer.audio.AudioRendererEventListener
import androidx.media3.exoplayer.audio.AudioSink
import androidx.media3.exoplayer.audio.DefaultAudioSink
import androidx.media3.exoplayer.audio.MediaCodecAudioRenderer
import androidx.media3.exoplayer.audio.TeeAudioProcessor
import androidx.media3.exoplayer.mediacodec.MediaCodecSelector
import io.flutter.Log
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import io.flutter.plugin.common.MethodChannel.MethodCallHandler
import io.flutter.plugin.common.MethodChannel.Result
import java.util.ArrayList

@OptIn(UnstableApi::class)
/** AudiopcPlugin */
class AudiopcPlugin : FlutterPlugin, MethodCallHandler {
    /// The MethodChannel that will the communication between Flutter and native Android
    ///
    /// This local reference serves to register the plugin with the Flutter Engine and unregister it
    /// when the Flutter Engine is detached from the Activity
    private lateinit var channel: MethodChannel
    private lateinit var player: ExoPlayer
    val samplesProcessor = SamplesProcessor()


    override fun onAttachedToEngine(flutterPluginBinding: FlutterPlugin.FlutterPluginBinding) {
        channel = MethodChannel(flutterPluginBinding.binaryMessenger, "audiopc")
        channel.setMethodCallHandler(this)
        initPlayer(flutterPluginBinding.applicationContext)
    }

    override fun onMethodCall(call: MethodCall, result: Result) {
        when (call.method) {
            "setVolume" -> {
                setVolume(call.argument<Double>("volume")!!)
                result.success(true)
            }

            "getVolume" -> {
                result.success(getVolume())
            }

            "setSource" -> {
                val path = call.argument<String>("path")
                if (path != null) {
                    setSource(path)
                } else {
                    result.error(
                        "SOURCE_NOT_FOUND",
                        "Source not found",
                        "Please provide a valid source"
                    )
                }
                result.success(path)
            }

            "play" -> {
                play()
                result.success(true)
            }

            "pause" -> {
                pause()
                result.success(true)
            }

            "getDuration" -> {
                result.success(getDuration())
            }

            "getSamples" -> {
                val samples = getSamples().toDoubleArray()
                result.success(samples)
            }

            "seek" -> {
                seek(call.argument<Double>("position")!!)
                result.success(0.0)
            }

            "getPosition" -> {
                result.success(getPosition())
            }

            "setRate" -> {
                setRate(call.argument<Double>("rate")!!)
                result.success(true)
            }

            "getState" -> {
                result.success(getState())
            }

            else -> {
                result.notImplemented()
            }
        }
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        channel.setMethodCallHandler(null)
    }

    private fun initPlayer(context: Context) {
        val defaultRenderersFactory = object : DefaultRenderersFactory(context) {
            override fun buildAudioRenderers(
                context: Context,
                extensionRendererMode: Int,
                mediaCodecSelector: MediaCodecSelector,
                enableDecoderFallback: Boolean,
                audioSink: AudioSink,
                eventHandler: Handler,
                eventListener: AudioRendererEventListener,
                out: ArrayList<Renderer>,
            ) {
                val teeAudioProcessor = TeeAudioProcessor(samplesProcessor)
                val defaultAudioSink = DefaultAudioSink.Builder(context).setAudioProcessors(
                    arrayOf(
                        teeAudioProcessor
                    )
                ).build()

                out.add(
                    MediaCodecAudioRenderer(
                        context,
                        mediaCodecSelector,
                        enableDecoderFallback,
                        eventHandler,
                        eventListener,
                        defaultAudioSink
                    )
                )
            }
        }
        player = ExoPlayer.Builder(context, defaultRenderersFactory).build()
    }

    private fun setVolume(volume: Double) {
        player.volume = volume.toFloat()
    }

    private fun getVolume(): Double {
        return player.volume.toDouble();
    }

    private fun setSource(path: String) {
        player.setMediaItem(MediaItem.fromUri(path))
    }

    private fun play() {
        player.play()
        player.prepare()
    }

    private fun pause() {
        player.pause()
    }

    private fun getDuration(): Double {
        return when (player.duration) {
            C.TIME_UNSET -> 0.0
            else -> player.duration.toDouble() / 1000.0
        }
    }

    private fun getSamples(): List<Double> {
       return samplesProcessor.samples
    }

    private fun seek(position: Double) {
        player.seekTo((position * 1000.0).toLong())
    }

    private fun getPosition(): Double {
        return when (player.currentPosition) {
            C.TIME_UNSET -> 0.0
            else -> player.currentPosition.toDouble() / 1000.0
        }
    }

    private fun setRate(rate: Double) {
        player.setPlaybackSpeed(rate.toFloat())
    }

    private fun getState(): Double {
        return if (player.isPlaying) {
            3.0
        } else if (player.playbackState == Player.STATE_ENDED) {
            return 5.0
        } else {
            4.0
        }
    }
}
