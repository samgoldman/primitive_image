# Primitive Image - Rust

[![Build Status](https://travis-ci.org/samgoldman/primitive_image.svg?branch=master)](https://travis-ci.org/samgoldman/primitive_image)

Approximate images using SVGs.

This is mostly a port of https://github.com/fogleman/primitive (written in Go), with a new thing or two. I encountered that project few years ago and recently used it to generate over a thousand SVG approximations as placeholders for an [image gallery](www.samueltgoldman.com/china) (while lazy loading the images). I decided I wanted to understand how it worked and I wanted to practice my Rust skills (this is my second project in the language), so I took about a week and wrote this.

I have implemented or will implement most features from the original project. Currently, my version can't create ellipses, although that is the next task.
It is also not able to create GIFs, or output images every n objects added. I don't currently have plans to add these functionalities, but they aren't out of the realm of possibilities.

## Usage

Download a binary from here: https://github.com/samgoldman/primitive_image/releases/.

From a terminal:

|Argument|Usage|
|---|---|
|-i, --input| (required) Path to the image to approximate. JPGs and PNGs are tested. TIFs, GIFs, and BMPs are theoretical.|
|-o, --output   | (required) Path to the output file. SVGs, JPGs, PNGs are tested.|
|-n   | (optional) The number of objects to use in the appoximation. Defaults to 100. Going about 1000 is pushing it.|
|--max-age|(optional) The maximum number of sequential failed mutations before adding an object. Defaults to 100. I have not tested above 500. |
|--scale-to|(optional) The number of pixels to scale the input image's largest side to before processing. Defaults to 100. Going above that really slows things down. |
|--seed|(optional) The seed for the random number generator. Defaults to 0, which sets the seed based on the time. If set, will result in repeatable outputs.|
|--shape|(optional) The shape to use for the approximations (TRIANGLE, RECTANGLE, ELLIPSE, QUADRATIC, CUBIC, MIXED). Defaults to TRIANGLE. (ELLIPSE and MIXED are not currently supported).|
|--background-color| (optional) The initial background color in RRGGBB format. Defaults to the average color of the input image. |

## Future work

- Add ellipses (and enable mixed approximations)
- Add progressive image saving
- Add GIF output
- Add generic polygons as an option
- Allow specification of mixed-image proportions by the user
- Allow multiple image output
- General optimization work
