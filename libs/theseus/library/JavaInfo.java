public final class JavaInfo {
    private static final String[] CHECKED_PROPERTIES = new String[] {
            "os.arch",
            "java.version"
    };

    public static void main(String[] args) {
        int returnCode = 0;

        for (String key : CHECKED_PROPERTIES) {
            String property = System.getProperty(key);

            if (property != null) {
                System.out.println(key + "=" + property);
            } else {
                returnCode = 1;
            }
        }

        System.exit(returnCode);
    }
}