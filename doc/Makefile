MD_TARGETS := $(wildcard *.md)
TEX_TARGETS := $(wildcard *.tex) $(MD_TARGETS:%.md=build/%.tex) 

.PHONY: build
build: build/main.pdf

.PHONY: clean
clean:
	rm -rf build _minted

.PHONY: cleanbuild
cleanbuild: clean build

# We assume all resources are used
build/main.pdf: $(TEX_TARGETS) build/pandoc-macros.tex $(wildcard res/*) bibliography.bib
	@mkdir -p $(@D)
	latexmk -bibtex -pdf -jobname=build/main main

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
	pandoc $< --filter res/pandoc-minted.py -o $@
