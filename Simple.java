public class Simple {

	public int posX;


	public Simple(int i) {
		posX = i;
	}


	public static void main(String[] args) {
		System.out.println("Test");

		Simple simple = new Simple(45);
		System.out.println(simple.posX);

	}
}
