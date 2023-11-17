[Deutsche Version](README.de.md)

# Televido

**Televido** (“Television” in Esperanto) lets you livestream, search, play and download media from German-language public television services. It is powered by [MediathekViewWeb](https://mediathekviewweb.de/)'s API and the [Zapp backend](https://github.com/mediathekview/zapp-backend) API which are both part of the [MediathekView](https://mediathekview.de/) project.

The presented content is provided directly by the respective television services, this program only facilitates finding and accessing the shows.

For video playback and download, Televido uses external programs that are installed on the user's system. Currently supported players: [GNOME Videos (Totem)](https://flathub.org/apps/org.gnome.Totem), [Celluloid](https://flathub.org/apps/io.github.celluloid_player.Celluloid), [Clapper](https://flathub.org/apps/com.github.rafostar.Clapper), [Daikhan](https://flathub.org/apps/io.gitlab.daikhan.stable). Currently supported downloaders: [Parabolic](https://flathub.org/apps/org.nickvision.tubeconverter).

## Channel logos

The ARD, ORF and SRF logos were taken from [Wikimedia Commons](https://commons.wikimedia.org) and are in the public domain.

The other channel logos were extracted from the source code of [zapp](https://github.com/mediathekview/zapp) and converted to SVG using [`vd2svg`](https://github.com/seanghay/vector-drawable-svg).

## FAQ

### How can I use a different video player / use a player with custom options?

Televido supports any video player that is [DBus activatable](https://specifications.freedesktop.org/desktop-entry-spec/latest/ar01s08.html) and supports opening https:// URIs via the `org.freedesktop.Application.Open` DBus method.

To use a custom player, create a flatpak permission override to allow it to access the player. E.g.

```
flatpak override --user de.k_bo.Televido --talk-name=org.example.VideoPlayer
```

and set the video player in the preferences.

If you want to use program that doesn't support DBus activation, you can create a wrapper script. See [d-k-bo/dbus-activatable-wrapper](https://github.com/d-k-bo/dbus-activatable-wrapper).

### Could you add support for TV channels from other countries?

Since this project is basically a client for [MediathekView](https://mediathekviewweb.de/), it's limited to the channels supported by them. The upstream project is focused on German content and is developed in German, so I doubt that there are any plans for non-German content.
ORF (Austrian TV) & SRF (Swiss TV) are supported though.

## License

Copyright (C) 2023 David Cabot

This program is free software; you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation; either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
