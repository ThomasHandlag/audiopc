package com.example.audiopc_example

import android.content.Context
import androidx.annotation.NonNull
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel.MethodCallHandler
import io.flutter.plugin.common.MethodChannel.Result
import com.thugbn.audiopc.AudiopcPlugin

class MainActivity : FlutterActivity() {
    override fun configureFlutterEngine(
        @NonNull flutterEngine: FlutterEngine,
    ) {
        flutterEngine.plugins.add(AudiopcPlugin())
        super.configureFlutterEngine(flutterEngine)
    }
}
