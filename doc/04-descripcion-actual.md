\chapter{Descripción de la situación actual}

<!--
  Se debe describir en este apartado un estudio de mercado o un estado del
  arte, en función de la tipología de proyecto. Se documentarán otras
  aplicaciones que existen actualmente o han existido en el mercado (realizando
  una breve historia de la evolución tecnológica) que realicen funcionalidades
  iguales o parecidas a las que se propone desarrollar en el TFG.

  Se expondrán posibles alternativas justificando la opción o el camino elegido
  para llevar a cabo el TFG. Este apartado debe contar con un grueso de
  referencias bibliográficas para dar validez a las afirmaciones que el autor
  expone.
-->

A continuación se exponen varias tecnologías en uso o desarrollo relacionadas
con el proyecto, divididas en aplicaciones orientadas a dispositivos comunes
(basadas en Wifi Direct o Bluetooth), software que necesita hardware menos
común como radios (y por lo tanto dirigido a usuarios más especializados),
y librerías / SDKs dirigidas a desarrolladores.

Finalmente, se hace una revisión a la implementación de Wifi Direct usada por
Android y Linux, ya que durante este trabajo he usado dicha tecnología para el
prototipo.

# Aplicaciones móviles descentralizadas

Algunas aplicaciones móviles han explorado activamente el uso de tecnologías de
red directa para proporcionar comunicación descentralizada, resiliente y, en
muchos casos, privada. La mayoría de los casos de uso están 

*FireChat*, desarrollada por Open Garden, fue una de las primeras aplicaciones
populares en usar Wi-Fi Direct y Bluetooth para crear redes mesh entre
teléfonos móviles, especialmente útil durante protestas o en lugares sin
conexión. Permitía enviar mensajes entre usuarios cercanos sin conexión a
internet. Su uso se hizo notable durante protestas como las de Hong Kong en
2014 \cite{firechat-2014-protests}. Aunque fue descontinuada en 2018, demostró
la viabilidad de estas tecnologías para comunicación temporal.

Otras aplicaciones similares están en uso, como *Briar* \cite{briar-project},
que funciona mediante Wi-Fi Direct, Bluetooth o Tor, o *Bridgefy*
\cite{bridgefy} (a pesar de múltiples problemas de seguridad
\cite{bridgefy-sec-1} \cite{bridgefy-sec-2}).

Recientemente, una aplicación similar llamada BitChat \cite{bitchat}, creada
for Jack Dorsey ha ganado tracción gracias a la popularidad de su creador. Su
whitepaper \cite{bitchat-whitepaper} describe como usan el protocolo Noise
\cite{noise-protocol} para garantizar la confidencialidad.

# Comunicaciones P2P basadas en radio

Meshtastic \cite{meshtastic} es un proyecto open source que utiliza
dispositivos con radios \gls{LoRa} para transmitir mensajes de texto entre
usuarios sin internet. Su bajo consumo de energía y gran alcance lo hacen ideal
para senderismo, comunidades rurales, o situaciones de emergencia. Cada nodo
funciona como repetidor dentro de una red mesh.

Aparte de este proyecto, existe mucha industria militar basada en este tipo de
tecnología para crear MANETs \cite{finabel-manets-overview}.

# Librerías y APIs de alto nivel

Con el objetivo de simplificar la integración de comunicación P2P en
aplicaciones, diversas plataformas han desarrollado APIs de alto nivel que
abstraen los detalles técnicos subyacentes.

## Google Nearby

*Google Nearby Connections API* permite crear redes de comunicación directa
entre dispositivos Android usando una combinación de Bluetooth, BLE, Wi-Fi y
Wi-Fi Direct. Proporciona detección de proximidad, establecimiento de conexión
y transferencia de datos bajo un modelo pub-sub \cite{google-nearby}.

## Multipeer Connectivity Framework

*Multipeer Connectivity Framework*, de Apple, ofrece capacidades similares en
iOS/macOS, permitiendo descubrir dispositivos cercanos y establecer sesiones
seguras de comunicación, sin necesidad de servidores externos \cite{multipeer-connectivity}.

## Menciones ilustres

Dentro del ecosistema de comunicación peer-to-peer, existen algunas librerías
con funcionalidad interesante, pero que no son directamente comparables a lo
que queremos hacer, porque no se encargan del descubrimiento de dispositivos,
es decir, tienes que saber por adelantado el dispositivo al que te quieres
conectar de otra forma.

### LibP2P

*libp2p*, parte del ecosistema de IPFS y utilizado en protocolos relacionados con
criptomonedas, ofrece una pila modular para comunicación P2P \cite{libp2p}.

Está más centrado en redes distribuidas a gran escala, sus conceptos de
transporte y autenticación también son relevantes para entornos sin
infraestructura.

### iroh

*iroh* \cite{iroh} es una librería similar para establecer comunicación directa
y un canal via \gls{QUIC} entre dispositivos a través de internet. Dadas dos
direcciones IP, trata de conectarlas directamente, haciendo \gls{hole-punching}
y fallback a un relay si es necesario.
