MD_TARGETS := $(wildcard *.md)
TEX_TARGETS := $(wildcard *.tex) $(MD_TARGETS:%.md=build/%.tex) 

# We assume all resources are used
build/main.pdf: $(TEX_TARGETS) build/pandoc-macros.tex $(wildcard res/*)
	@mkdir -p $(@D)
	xelatex -shell-escape -output-directory=build main.tex
	xelatex -shell-escape -output-directory=build main.tex

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

build/%.tex: %.md
	@mkdir -p $(@D)
	pandoc $< -o $@
