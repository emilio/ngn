# https://mirrors.ctan.org/support/latexmk/example_rcfiles/glossaries_latexmkrc
add_cus_dep( 'acn', 'acr', 0, 'makeglossaries' );
add_cus_dep( 'glo', 'gls', 0, 'makeglossaries' );
$clean_ext .= " acr acn alg glo gls glg";

sub makeglossaries {
  my @args = ( "-s", "$_[0].ist", "-t", "$$Psource.ilg",
               "-o", $$Pdest, $$Psource );
  if ($silent) { unshift @args, "-q"; }
  return system "makeindex", @args;
}
