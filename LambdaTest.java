import java.util.function.Consumer;

public class LambdaTest {

    public static void apply(Consumer consumer) {
        consumer.accept("Foo");
    }

    public static void main(String[] args) {
        apply(x -> System.out.println(x + " Bar"));
    }
}
