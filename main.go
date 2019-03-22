package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"strings"

	"github.com/bendahl/uinput"
	evdev "github.com/gvalkov/golang-evdev"
)

func main() {
	device := flag.String("device", "", "path to keyboard character device. example: /dev/input/event42")
	keymapStr := flag.String("keymap", "", "example: LeftShift=Escape,RightCtrl=End")

	flag.Parse()

	if *device == "" || *keymapStr == "" {
		flag.PrintDefaults()
		os.Exit(1)
	}

	keymap, err := parseKeymap(*keymapStr)

	if err != nil {
		log.Fatal(err)
	}

	if err := handleDevice(*device, keymap); err != nil {
		log.Fatal(err)
	}
}

func parseKeymap(s string) (map[int]int, error) {
	keymap := make(map[int]int)

	for _, entry := range strings.Split(s, ",") {
		kv := strings.SplitN(entry, "=", 2)

		found, k := findKeyCodeByName(kv[0])

		if !found {
			return nil, fmt.Errorf("unknown key name: %s", kv[0])
		}

		found, v := findKeyCodeByName(kv[1])

		if !found {
			return nil, fmt.Errorf("unknown key name: %s", kv[1])
		}

		keymap[k] = v
	}

	return keymap, nil
}

func findKeyCodeByName(name string) (bool, int) {
	name = "KEY_" + strings.ToUpper(strings.TrimSpace(name))

	for k, v := range evdev.KEY {
		if v == name {
			return true, k
		}
	}

	return false, 0
}

func handleDevice(path string, keymap map[int]int) error {
	reader, err := evdev.Open(path)

	if err != nil {
		return err
	}

	if err := reader.Grab(); err != nil {
		return err
	}

	defer reader.Release()

	writer, err := uinput.CreateKeyboard("/dev/uinput", []byte(reader.Name+" [osm]"))

	if err != nil {
		return err
	}

	defer writer.Close()

	holdKey := evdev.KEY_UNKNOWN

	for {
		events, err := reader.Read()

		if err != nil {
			return err
		}

		for _, ev := range events {
			holdKey = handleKeyEvent(ev, writer, keymap, holdKey)
		}
	}
}

func handleKeyEvent(ev evdev.InputEvent, writer uinput.Keyboard, keymap map[int]int, holdKey int) int {
	if ev.Type != evdev.EV_KEY {
		return holdKey
	}

	key := int(ev.Code)
	extraKey, isTrigger := keymap[key]

	switch evdev.KeyEventState(ev.Value) {
	case evdev.KeyDown:
		if holdKey != evdev.KEY_UNKNOWN {
			writer.KeyDown(holdKey)
		}

		if isTrigger {
			return key
		}

		writer.KeyDown(key)

		return evdev.KEY_UNKNOWN
	case evdev.KeyUp:
		if key == holdKey {
			writer.KeyPress(extraKey)
		} else {
			writer.KeyUp(key)
		}

		return evdev.KEY_UNKNOWN
	}

	return holdKey
}
