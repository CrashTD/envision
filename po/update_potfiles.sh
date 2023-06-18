#!/bin/bash

APPID="org.gabmus.rex2"

rm po/POTFILES.in po/POTFILES.in.in

{
    grep -Fl "gettext(\"" src/**.rs
    # grep -Fl "_(\"" data/resources/ui/**.blp
    echo "data/${APPID}.desktop.in.in"
    echo "data/${APPID}.metainfo.xml.in.in"
} >> po/POTFILES.in.in

sort < po/POTFILES.in.in | uniq > po/POTFILES.in
rm po/POTFILES.in.in
