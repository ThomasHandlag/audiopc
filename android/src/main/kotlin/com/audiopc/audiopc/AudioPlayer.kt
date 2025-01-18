package com.audiopc.audiopc

import android.content.Context
import android.os.Handler
import androidx.annotation.OptIn
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.MediaMetadata
import androidx.media3.common.PlaybackException
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
import io.flutter.plugin.common.EventChannel
import java.util.ArrayList

class AudioPlayer @OptIn(UnstableApi::class) constructor(
    private var id: String,
    private var eventSink: EventChannel.EventSink,
    context: Context
) {
    private var player: ExoPlayer
    private lateinit var samplesProcessor: SamplesProcessor

    init {
        val defaultRenderersFactory =
            object : DefaultRenderersFactory(context) {
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
                    samplesProcessor = SamplesProcessor()
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
        player.addListener(object : Player.Listener {
            override fun onPlaybackStateChanged(unused: Int) {
                var state = 0
                when (player.playbackState) {
                    Player.STATE_IDLE -> state = 0
                    Player.STATE_BUFFERING -> state = 1
                    Player.STATE_READY -> state = 2
                    Player.STATE_ENDED -> state = 5
                }

                if (player.isPlaying) {
                    state = 3
                } else if (player.isPlaying) {
                    state = 4
                }

                eventSink.success(
                    mapOf(
                        "id" to id,
                        "event" to "state",
                        "value" to state
                    )
                )
            }

            override fun onMediaMetadataChanged(mediaMetadata: MediaMetadata) {
                super.onMediaMetadataChanged(mediaMetadata)
                eventSink.success(
                    mapOf(
                        "id" to id,
                        "event" to "metadata",
                        "value" to mediaMetadata.durationMs
                    )
                )
            }

            override fun onPlayerError(error: PlaybackException) {
                eventSink.success(
                    mapOf(
                        "id" to id,
                        "event" to "error",
                        "value" to error.message
                    )
                )
            }

            override fun onEvents(player: Player, events: Player.Events) {
                super.onEvents(player, events)
                if (player.playbackState == Player.STATE_ENDED) {
                    eventSink.success(
                        mapOf(
                            "id" to id,
                            "event" to "completed",
                            "value" to 1
                        )
                    )

                }
            }
        })
    }

    fun setVolume(volume: Double) {
        player.volume = volume.toFloat()
    }

    fun setRate(rate: Double) {
        player.setPlaybackSpeed(rate.toFloat())
    }

    fun seek(position: Double) {
        player.seekTo((position * 1000.0).toLong())
    }

    fun getPosition(): Double {
        return when (player.currentPosition) {
            C.TIME_UNSET -> 0.0
            else -> player.currentPosition.toDouble() / 1000.0
        }
    }

    fun shutdown() {
        player.stop()
        player.release()
    }

    fun setSource(path: String) {
        player.setMediaItem(MediaItem.fromUri(path))
    }

    fun play() {
        player.play()
        player.prepare()
    }

    fun pause() {
        player.pause()
    }

    fun getSamples(): List<Double> {
        return samplesProcessor.samples
    }
}