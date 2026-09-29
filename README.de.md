<!-- SPDX-FileCopyrightText: David Cabot <d-k-bo@mailbox.org> -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

[English Version](README.md)

# Televido

> [!WARNING]
> ## Dieses Projekt ist eingestellt
>
> ### Warum?
>
> Als Hobby-Programmierer habe ich eine beträchtliche Menge an persönlicher Zeit und Energie in viele Projekte investiert. Aufgrund zunehmender privater und beruflicher Verpflichtungen habe ich jedoch deutlich weniger Zeit, die ich in die Softwareentwicklung stecken kann. Daher möchte ich mich lieber auf die Projekte konzentrieren, die ich selbst aktiv nutze und bei denen mir die Arbeit am meisten Spaß macht. Darüber hinaus verspüre ich insgesamt durch den aktuellen Einfluss von „KI“ einen leichten Interessensverlust an Softwareentwicklung.
>
> ### Was bedeutet das?
>
> - Dieses Projekt wird auf absehbare Zeit nicht mehr gepflegt und sollte dementsprechend als veraltet (deprecated) betrachtet werden.
> - Das Projekt bleibt weiterhin gemäß den Bedingungen der entsprechenden Frei-Software-Lizenz verfügbar.
> - Gerne kann es unter den Bedingungen der genannten Lizenz weiterverbreitet, geforkt und weiterentwickelt werden.


**Televido** („Fernsehen“ auf Esperanto) ermöglicht das Empfangen von Livestreams sowie das Suchen, Abspielen und Herunterladen von Inhalten aus Mediatheken öffentlich-rechtlicher Sender aus dem DACH-Raum. Es basiert auf den APIs von [MediathekViewWeb](https://mediathekviewweb.de/) und des [Zapp-Backends](https://github.com/mediathekview/zapp-backend), die beide Teil des [MediathekView](https://mediathekview.de/)-Projekts sind.

Die präsentierten Inhalte werden direkt von den jeweiligen Sendern angeboten, dieses Programm erleichtert dabei nur das Auffinden und Abrufen der Sendungen.

Televido bietet einen integrierten Videoplayer zum Abspielen der Inhalte. Darüber hinaus unterstützt Televido für Videowiedergabe und -download externe Programme, die auf dem System installiert sind. Derzeit unterstütze Player: [GNOME Videos (Totem)](https://flathub.org/apps/org.gnome.Totem), [Celluloid](https://flathub.org/apps/io.github.celluloid_player.Celluloid), [Clapper](https://flathub.org/apps/com.github.rafostar.Clapper), [Daikhan](https://flathub.org/apps/io.gitlab.daikhan.stable). Derzeit unterstütze Videodownloader: [Parabolic](https://flathub.org/apps/org.nickvision.tubeconverter).

## Senderlogos

Die ARD-, ORF- and SRF-Logos wurden [Wikimedia Commons](https://commons.wikimedia.org) entnommen und sind gemeinfrei.

Die anderen Logos wurden aus dem Quellcode von [zapp](https://github.com/mediathekview/zapp) extrahiert und mit [`vd2svg`](https://github.com/seanghay/vector-drawable-svg) ins SVG-Format konvertiert.

## FAQ

Siehe [README.md#faq](README.md#faq) (Englisch)

## Lizenz

Copyright (C) 2023 David Cabot

Dieses Programm ist freie Software. Sie können es unter den Bedingungen der GNU General Public License, wie von der Free Software Foundation veröffentlicht, weitergeben und/oder modifizieren, entweder gemäß Version 3 der Lizenz oder (nach Ihrer Option) jeder späteren Version.

Die Veröffentlichung dieses Programms erfolgt in der Hoffnung, daß es Ihnen von Nutzen sein wird, aber OHNE IRGENDEINE GARANTIE, sogar ohne die implizite Garantie der MARKTREIFE oder der VERWENDBARKEIT FÜR EINEN BESTIMMTEN ZWECK. Details finden Sie in der GNU General Public License.
