#include <dispatch/dispatch.h>
#include <xpc/xpc.h>

int main()
{
    xpc_connection_t conn = xpc_connection_create_mach_service(
        "com.example.echo", NULL, XPC_CONNECTION_MACH_SERVICE_LISTENER);

    xpc_connection_set_event_handler(conn, ^(xpc_object_t peer) {
        xpc_connection_set_event_handler(peer, ^(xpc_object_t event) {
            if (event == XPC_ERROR_CONNECTION_INVALID) {
                printf("Connection closed by remote end\n");
                return;
            }

            if (xpc_get_type(event) != XPC_TYPE_DICTIONARY) {
                printf("Didn't receive a dictionary\n");
                return;
            }

            xpc_connection_send_message(peer, event);
        });

        xpc_connection_resume(peer);
    });

    xpc_connection_resume(conn);
    dispatch_main();
}
