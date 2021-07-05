
public class ArrayTest {


	public static void printInfo(Object obj) {
		System.out.println(obj.getClass());
	}

	public static void main(String[] args) {
		int[] a = new int[6];
		Object[] b = new Object[2];
		printInfo(a);
		printInfo(b);
	}


}
