#### For Black Magic Probe:

# target extended-remote /dev/ttyACM0
# monitor swdp_scan
# att 1
# set print asm-demangle on
# load
# step


#### For OpenOCD:

target remote :3333
set print asm-demangle on
monitor arm semihosting enable
load
step
