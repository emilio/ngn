MD_TARGETS := $(wildcard *.md)
MERMAID_TARGETS := $(wildcard images/*.mmd)
TEX_TARGETS := $(wildcard *.tex) $(MD_TARGETS:%.md=build/%.tex) 
PDF_TARGETS := $(wildcard images/*.pdf) $(MERMAID_TARGETS:%.mmd=build/%.pdf)

.PHONY: build
build: build/main.pdf

.PHONY: clean
clean:
	rm -rf build _minted

.PHONY: cleanbuild
cleanbuild: clean build

# We assume all resources are used
build/main.pdf: $(TEX_TARGETS) $(PDF_TARGETS) build/pandoc-macros.tex $(wildcard res/*) bibliography.bib
	@mkdir -p $(@D)
	latexmk -shell-escape -bibtex -pdf -jobname=build/main main

# This is a bit hacky. Pandoc generates latex output but relies on a set of
# macros to be present on the document via $highlighting-macros$.
#
# In order to avoid going full pandoc by creating a custom template or what not
# (which honestly maybe wasn't the worst?) we extract those macros separately.
#
# There should, arguably, be a better way to do this.
build/pandoc-macros.tex: res/pandoc-macros.md res/pandoc-macros.tex.in
	@mkdir -p $(@D)
	pandoc $< --template=res/pandoc-macros.tex.in -o $@

build/%.tex: %.md res/pandoc-minted.py
	@mkdir -p $(@D)
	pandoc --table-caption-position=below $< --filter res/pandoc-minted.py -o $@

build/%.pdf: %.mmd
	@mkdir -p $(@D)
	mmdc --width 600 --height 600 -i $< -o $@
