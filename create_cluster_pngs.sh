#!/bin/bash
for filename in clusters/*.dot; do
    dot -Tpng -o "clusters/$(basename "$filename" .dot).png" -Efontname=Roboto -Efontsize=8 -Epenwidth=2 -Npenwidth=2 < "$filename"
done

cd clusters 
rm *.gif
chmod u+x *.sh
for filename in *.sh; do
    ./$filename
done
cd ..
rm clusters/*.png
rm clusters/*.dot
rm clusters/*.sh

