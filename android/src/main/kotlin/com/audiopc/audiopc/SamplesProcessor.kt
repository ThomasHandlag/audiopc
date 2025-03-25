package com.audiopc.audiopc

import androidx.annotation.OptIn
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.audio.TeeAudioProcessor.AudioBufferSink
import java.nio.ByteBuffer
import java.nio.ShortBuffer

@OptIn(UnstableApi::class)
class SamplesProcessor : AudioBufferSink {
    var samples = ArrayList<Double>()
    private var sampleRate = 44800
    private var channelCount = 2

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
        val leftChannel = ShortBuffer.allocate(numSamples)

        for (i in 0 until numSamples) {
            leftChannel.put(samples[i * 2])
        }
        leftChannel.flip()
        return leftChannel
    }
}