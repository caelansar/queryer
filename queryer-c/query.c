#include <stdio.h>
#include <stdlib.h>
#include "bindings.h"

int main() {
    const char name[] = "cae";
    printf("%s\n", hello(name));

    const char sql[] = "select * from file://../queryer-rs/examples/data.json";
    char *result = query(sql, NULL);
    printf("%s\n", result);

    free_str(result);

    return 0;
}
