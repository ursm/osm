.ONESHELL:
.SHELLFLAGS := -ec
.PHONY: all build install README.md

INSTALL ?= install

CARGO_TARGET_DIR ?= target

PREFIX       ?= /usr/local
BINDIR       ?= $(PREFIX)/bin
SYSCONFDIR   ?= /etc
UNITDIR      ?= $(PREFIX)/lib/systemd/system
UDEVRULESDIR ?= $(PREFIX)/lib/udev/rules.d

all: build

build:
	cargo build --release

# Deliberately does not depend on `build`: this is the target you run under
# sudo, and building as root breaks a rustup toolchain and leaves the target
# directory owned by root. Run `make` first.
install:
	$(INSTALL) -D -m 755 $(CARGO_TARGET_DIR)/release/osm $(DESTDIR)$(BINDIR)/osm
	$(INSTALL) -D -m 644 dist/udev/99-osm.rules          $(DESTDIR)$(UDEVRULESDIR)/99-osm.rules

	# The unit has to name the paths this Makefile was configured with, so it is
	# generated here rather than shipped ready to copy.
	mkdir -p $(DESTDIR)$(UNITDIR)
	sed -e 's|@BINDIR@|$(BINDIR)|g' -e 's|@SYSCONFDIR@|$(SYSCONFDIR)|g' dist/systemd/osm@.service.in > $(DESTDIR)$(UNITDIR)/osm@.service
	chmod 644 $(DESTDIR)$(UNITDIR)/osm@.service

	# Never clobber key mappings someone has already set.
	if [ ! -e $(DESTDIR)$(SYSCONFDIR)/default/osm ]; then
	  $(INSTALL) -D -m 644 dist/default/osm $(DESTDIR)$(SYSCONFDIR)/default/osm
	fi

README.md: README.md.hms
	handlematters $< > $@.tmp
	mv $@.tmp $@
