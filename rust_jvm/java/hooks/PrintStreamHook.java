package java.hooks;

import java.io.IOException;
import java.io.OutputStream;
import java.io.PrintStream;
import java.util.Objects;

public class PrintStreamHook extends OutputStream {
    private final int fd;

    private PrintStreamHook(int fd) {
        this.fd = fd;
    }

    public static PrintStream buildStream(int fd) {
        return new PrintStream(new PrintStreamHook(fd), true);
    }

    private native void sendIO(int fd, String text);

    @Override
    public void write(byte[] b, int off, int len) throws IOException {
        if (off + len > b.length || off < 0) {
            throw new IllegalArgumentException();
        }
        sendIO(fd, new String(b, off, len));
    }

    @Override
    public void write(int b) throws IOException {
        write(new byte[] {(byte) b});
    }
}
