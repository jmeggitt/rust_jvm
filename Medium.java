import java.io.IOException;
import java.util.Properties;

public class Medium {
    public static void main(String[] args) throws IOException {
        Properties properties = System.getProperties();
        System.out.println(System.mapLibraryName("awt"));
    }
}