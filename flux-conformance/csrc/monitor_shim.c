/* monitor_shim.c — C-monitor runner shim for the conformance harness.
 *
 * flux_monitor_arm.c's API exchanges the 9KB `flux_monitor` struct by
 * pointer, which is awkward to mirror byte-exactly from Rust. This shim
 * owns the struct on the heap and exposes an opaque-pointer API instead.
 * Built with the same -Werror flags the repo CI uses for the C sources.
 */
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>

#include "flux_monitor_arm.c"

void* conf_monitor_new(void) {
    return calloc(1, sizeof(flux_monitor));
}

bool conf_monitor_load(void* p, const uint8_t* bc, uint16_t len) {
    flux_monitor_init((flux_monitor*)p);
    return flux_monitor_load((flux_monitor*)p, bc, len);
}

bool conf_monitor_run(void* p) {
    return flux_monitor_run((flux_monitor*)p);
}

uint8_t conf_monitor_error(void* p) {
    return flux_monitor_get_error((flux_monitor*)p);
}

uint32_t conf_monitor_passed(void* p) {
    return flux_monitor_get_passed((flux_monitor*)p);
}

uint32_t conf_monitor_failed(void* p) {
    return flux_monitor_get_failed((flux_monitor*)p);
}

void conf_monitor_free(void* p) {
    free(p);
}
