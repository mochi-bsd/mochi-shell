# Hanami Compositor Makefile

CC = gcc
CFLAGS = -Wall -Wextra -std=c11 -DWLR_USE_UNSTABLE \
         $(shell pkg-config --cflags wlroots wayland-server xkbcommon)
LDFLAGS = $(shell pkg-config --libs wlroots wayland-server xkbcommon)

TARGET = hanami
SOURCES = main.c
OBJECTS = $(SOURCES:.c=.o)

all: $(TARGET)

$(TARGET): $(OBJECTS)
	$(CC) $(OBJECTS) -o $(TARGET) $(LDFLAGS)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(OBJECTS) $(TARGET)

install: $(TARGET)
	install -D -m 755 $(TARGET) $(DESTDIR)/usr/local/bin/$(TARGET)

.PHONY: all clean install
