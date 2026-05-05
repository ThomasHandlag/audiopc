package com.thugbn.audiopc;

import android.content.Context;

public final class AudiopcBridge {
    static {
        System.loadLibrary("audiopc");
    }

    private AudiopcBridge() {}

    public static void init(Context context) {
        nativeInit(context.getApplicationContext());
    }

    private static native void nativeInit(Context context);
}
