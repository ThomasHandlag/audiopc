package com.audiopc.audiopc

import androidx.annotation.OptIn
import androidx.media3.common.audio.BaseAudioProcessor
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.audio.TeeAudioProcessor.AudioBufferSink
import java.nio.ByteBuffer
import java.nio.ShortBuffer

@OptIn(UnstableApi::class)
class SamplesProcessor : AudioBufferSink {
    var samples = ArrayList<Double>()
    var sampleRate = 44100
    var channelCount = 2

    override fun flush(sampleRateHz: Int, channelCount: Int, encoding: Int) {
        this.sampleRate = sampleRateHz
        this.channelCount = channelCount
    }

    override fun handleBuffer(buffer: ByteBuffer) {
        val samples = leftSamples(buffer.asShortBuffer())
        this.samples = ArrayList(samples.remaining())
        while (samples.hasRemaining()) {
            this.samples.add(samples.get().toDouble())
        }
    }

    private fun leftSamples(samples: ShortBuffer): ShortBuffer {
        val numSamples = samples.limit() / 2  // 2 channels, 2 bytes per channel

        // Create output arrays for left and right channels
        // Allocate buffers for left and right channels
        val leftChannel = ShortBuffer.allocate(numSamples)
//        val rightChannel = ShortBuffer.allocate(numSamples)

        // Process interleaved stereo data
        for (i in 0 until numSamples) {
            // Interleaved: [Left1, Right1, Left2, Right2, ...]
            leftChannel.put(samples[i * 2])       // Left channel sample
//            rightChannel.put(samples[i * 2 + 1]) // Right channel sample
        }

        // Reset positions for read access
        leftChannel.flip()
//        rightChannel.flip()

        return leftChannel
    }
}