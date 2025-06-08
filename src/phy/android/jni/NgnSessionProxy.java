/* This class is a proxy between the Rust side P2PSession and the native
 * WifiP2PManager */
public class NgnSessionProxy {
    private static native long ngn_session_init(NgnSessionProxy session);

    static {
        System.loadLibrary("ngn");
    }

    public NgnSessionProxy(Object p2pManager) {
        this.m_manager = p2pManager;
        this.m_native = ngn_session_init(this);
    }

    Object m_manager;
    long m_native;
}
