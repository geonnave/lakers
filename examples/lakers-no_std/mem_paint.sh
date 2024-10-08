#!/bin/bash

# first, check it compiles
if ! cargo build --target=thumbv7em-none-eabihf 2>/dev/null; then
    echo "Compilation failed"
    exit 1
fi
echo "Compilation successful"

# get the size of the data and bss sections
data_size=$(cargo size --target=thumbv7em-none-eabihf 2>/dev/null | tail -1 | awk '{print $2}')
bss_size=$(cargo size --target=thumbv7em-none-eabihf 2>/dev/null | tail -1 | awk '{print $3}')

# update it in the code
# fix the line static DATA_SIZE: usize = 56;
sed -i "s/static DATA_SIZE: usize = [0-9]\+;/static DATA_SIZE: usize = $data_size;/" src/main.rs
# sed -i "s/static BSS_SIZE: usize = [0-9]\+;/static BSS_SIZE: usize = $bss_size;/" src/main.rs
echo "Updated data and bss sizes"

# flash and get the amount of bytes painted
echo "Flashing..."
painted=$(cargo run --target=thumbv7em-none-eabihf 2>/dev/null | grep -o "total of.*bytes" | head -1 | egrep -o '[0-9]+')
printf "\nPainted $painted bytes\n"

# divide by 4 to get the number of words
words=$((painted / 4))

# address but offset by the data section
base_address=$((0x20000000 + data_size))

# get the amount of memory used
# example concrete command: probe-rs read b32 0x20000080 434 --chip nRF52840_xxAA | tr ' ' '\n' | grep deadbeef | wc -l | awk '{print $1*4}'
remaining=$(
    probe-rs read b32 $base_address $words --chip nRF52840_xxAA | \
    tr ' ' '\n' | \
    grep deadbeef | \
    wc -l | \
    awk '{print $1*4}'
)
used=$((painted - remaining))
printf "Used $used bytes\n"
