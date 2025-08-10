\chapter{Antecedentes}

<!--
  Se deben presentar aquí de forma breve los aspectos teóricos previos del
  problema que no tengan que ver con el aspecto técnico de la informática. Por
  ejemplo, si el trabajo tiene que ver con la didáctica, se debe hablar de este
  campo, si tiene que ver con el cuidado de mayores, se debe hablar de la
  problemática de esta área, etc.

  También se incluyen la revisión de estudios, investigaciones previas o
  cualquier información relevante que ayuda a contextualizar el tema del TFG.
  En cualquier caso, los antecedentes brindan una base teórica o empírica que
  permite entender el estado del conocimiento sobre el tema, identificar vacíos
  o limitaciones en estudios anteriores, y justificar la necesidad de realizar
  el nuevo estudio.

  Si el TFG es de naturaleza investigadora, los antecedentes ayudan a
  fundamentar el problema de investigación y orientan el enfoque del
  investigador en su trabajo.

  Si se ha realizado el TFG en el ámbito de una empresa, y con el
  consentimiento de dicha empresa, en esta sección se puede presentar su
  trayectoria, historia, productos que desarrolla, trabajos más
  representativos, etc. El objetivo es realizar un marco contextual en el que
  se engloba el trabajo.
-->

# Acceso a Internet, centralización y privacidad

Aunque el acceso a internet ha crecido exponencialmente desde finales del siglo
XX, su disponibilidad sigue siendo desigual. Según datos de la Unión
Internacional de Telecomunicaciones, a finales de 2024, aproximadamente
un 30% de la población mundial aún no tenía acceso regular a internet
\cite{itu-2024}. Esta brecha digital afecta especialmente a comunidades
rurales, zonas en desarrollo y regiones afectadas por conflictos o desastres
naturales.

En sus inicios, Internet fue concebida como una red descentralizada de nodos
capaces de comunicarse directamente, diseñada para ser resiliente ante fallos
parciales. Sin embargo, con el paso de los años, la arquitectura de internet ha
evolucionado hacia un modelo cada vez más centralizado, donde gran parte del
tráfico global se canaliza a través de servidores propiedad de un número
reducido de grandes corporaciones tecnológicas \cite{rfc9518}.

Esta centralización tiene varias consecuencias preocupantes, tanto para la
resiliencia de las comunicaciones en situaciones de emergencia, conflictos o
censura, como para la privacidad, libertad de expresión, y autonomía de los
usuarios.

Frente a esta realidad, surgen soluciones que buscan recuperar la
descentralización original de internet, permitiendo a los dispositivos
comunicarse entre sí directamente, sin necesidad de depender de intermediarios
ni infraestructura externa. Estas alternativas permiten fomentar modelos de
comunicación más autónomos, resistentes y respetuosos con la privacidad.

# Infraestructura técnica: Redes ad-hoc y comunicación punto a punto

Las tecnologías de comunicación directa entre dispositivos --- como Bluetooth,
Wi-Fi Direct, y modos ad-hoc de Wi-Fi --- permiten establecer redes sin
infraestructura, también llamadas redes ad-hoc. Estas redes son especialmente
útiles en contextos donde no existe un punto de acceso centralizado, o donde se
desea evitar su uso por razones de privacidad o autonomía.

En el ámbito de la investigación, las redes malladas (mesh networks) han sido
ampliamente estudiadas como una solución escalable y auto-organizada para
conectar múltiples dispositivos sin necesidad de infraestructura fija
\cite{mesh-survey-akyildiz-2005}. Estas redes permiten que cada nodo actúe como
cliente y repetidor, lo que amplía el alcance de la red sin necesidad de un
servidor central.

La adopción de este tipo tecnologías ha sido limitada tanto por parte de usuarios
como de desarrolladores. Por ejemplo, aunque tecnologías como Wi-Fi Direct
están ampliamente disponibles a nivel de hardware, su implementación y uso en
software comercial sigue siendo marginal.

Desde una perspectiva sociotécnica, esta baja adopción también responde a
dinámicas de mercado. Las plataformas centralizadas ofrecen comodidad y
servicios integrados que han establecido una expectativa de funcionamiento
transparente, lo que dificulta que modelos descentralizados compitan en
términos de experiencia de usuario. Además, las grandes plataformas
no promueven activamente estas formas de comunicación, en parte porque reducen
su capacidad de control y monetización.

Por tanto, la brecha entre el potencial técnico y la adopción real de estas
tecnologías subraya la necesidad de herramientas que simplifiquen su uso,
abstraigan su complejidad, y proporcionen interfaces más accesibles para los
desarrolladores. Esta es precisamente la línea de trabajo en la que se enmarca
el presente proyecto.
