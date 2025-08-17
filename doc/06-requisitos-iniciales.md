\chapter{Requisitos iniciales}

<!--
  Se deberá presentar, según la metodología, un resumen de los requisitos
  aportados en el Anexo de "Especificaciones del sistema" mediante casos de
  uso, historias de usuario, etc.
-->

En este capítulo se recogen, de manera resumida, los requisitos del sistema,
especificados siguiendo la Metodología para la Elicitación de Requisitos de
Sistemas Software de Durán Toro y Bernárdez Jiménez \cite{req-duran-bernardez}.

En el *Anexo I. Especificaciones del sistema* se especifican estos requisitos
de manera más detallada, siguiendo, en la medida de lo posible, la estructura y
plantillas especificada por Durán y Bernárdez \cite{req-duran-bernardez}.

Este capítulo constituye un resumen de las partes más relevantes de dicho
anexo.

# Requisitos de información

Según la norma IEEE 29148-2018 sobre ingeniería de requisitos
\cite{ieee-29148}, los requisitos de información \enquote{definen los
requisitos para la gestión por parte del sistema de la información que recibe,
genera o exporta} (p. 66).

TODO

# Requisitos funcionales

Los requisitos funcionales definen \enquote{qué debe hacer el sistema con la información
almacenada para alcanzar los objetivos de su negocio}.

TODO

# Requisitos no funcionales
\label{sec:nfr}

Los requisitos no funcionales son aquellos que definen cualidades sobre el
sistema que no están directamente relacionadas con la funcionalidad del mismo,
sino con aspectos como el rendimiento, la usabilidad, la seguridad, etc
\cite{penalvo-requirements}.

Así, se han definido los siguientes requisitos no funcionales para el sistema:

  1. **Portabilidad**: Accesibilidad desde diferentes plataformas y sistemas operativos.
  2. **Extensibilidad**: Facilidad para añadir nuevas plataformas y transportes físicos.
  3. **Seguridad**: Identificación y cifrado de mensajes independiente de la capa física.
  4. **Accesibilidad**: La biblioteca deberá funcionar con dispositivos
     accesibles / no requerir hardware especial.
