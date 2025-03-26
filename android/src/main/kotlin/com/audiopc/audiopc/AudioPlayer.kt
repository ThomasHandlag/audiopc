package com.audiopc.audiopc

import android.content.Context
import android.os.Handler
import androidx.annotation.OptIn
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.common.Timeline
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
import io.flutter.plugin.common.EventChannel
import java.util.ArrayList

@OptIn(UnstableApi::class)
class AudioPlayer(
    private var id: String,
    private var eventSink: EventChannel.EventSink,
    context: Context,
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
                Log.d("AudioPlayer", "State: $unused: ${player.duration}")
                when (unused) {
                    Player.STATE_READY-> {
                        eventSink.success(
                            mapOf(
                                "id" to id,
                                "event" to "state",
                                "value" to 2
                            )
                        )
                    }
                    else -> {
                        eventSink.success(
                            mapOf(
                                "id" to id,
                                "event" to "state",
                                "value" to 0
                            )
                        )

                    }
                }
            }

            override fun onIsPlayingChanged(isPlaying: Boolean) {
                super.onIsPlayingChanged(isPlaying)
                eventSink.success(
                    mapOf(
                        "id" to id,
                        "event" to "state",
                        "value" to if (isPlaying) 3 else 4
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

            override fun onTimelineChanged(timeline: Timeline, reason: Int) {
                super.onTimelineChanged(timeline, reason)
                when(player.duration) {
                    C.TIME_UNSET -> {
                        eventSink.success(
                            mapOf(
                                "id" to id,
                                "event" to "duration",
                                "value" to 0.0
                            )
                        )
                    }
                    else -> {
                        eventSink.success(
                            mapOf(
                                "id" to id,
                                "event" to "duration",
                                "value" to player.duration.toDouble() / 1000.0
                            )
                        )
                    }
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
            else -> player.currentPosition / 1000.0
        }
    }

    fun shutdown() {
        player.stop()
        player.release()
    }

    fun setSource(path: String) {
        player.setMediaItem(MediaItem.fromUri(path))
        player.prepare()
        player.play()
    }

    fun play() {
        player.play()
    }

    fun pause() {
        player.pause()
    }

    fun getSamples(): List<Double> {
        return samplesProcessor.samples
    }

    fun getMetaData(path: String) : Map<String, Any?> {
        val mediaItem = MediaItem.fromUri(path)
        val metadata = mediaItem.mediaMetadata
        Log.d("AudioPlayer", "Metadata: $metadata")
        return mapOf(
            "title" to metadata.title,
            "artist" to metadata.artist,
            "albumTitle" to metadata.albumTitle,
            "artwork" to metadata.artworkData,
            "timeReleased" to metadata.releaseDay,
            "copyRight" to metadata.conductor,
            "genre" to metadata.genre,
        )
    }
}