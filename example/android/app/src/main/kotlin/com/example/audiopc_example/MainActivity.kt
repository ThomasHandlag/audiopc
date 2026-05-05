package com.example.audiopc_example

import android.os.Bundle
import com.thugbn.audiopc.AudiopcBridge
import io.flutter.embedding.android.FlutterActivity

class MainActivity : FlutterActivity() {
	override fun onCreate(savedInstanceState: Bundle?) {
		super.onCreate(savedInstanceState)
		AudiopcBridge.init(applicationContext)
	}
}