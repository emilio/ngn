\chapter{Análisis y diseño del sistema}

La arquitectura de la biblioteca ha sido descrita en prosa en el
\cref{chap:solution}, por lo que se va a evitar reiterar sobre ello en exceso.

Sin embargo, se pueden detallar determinadas interacciones y relaciones con
diagramas más descriptivos, que ocupaban demasiado espacio en la memoria. A
continuación se proveen algunos de estos diagramas más descriptivos.

# Diagrama de componentes

A continuación se muestra la relación entre los principales componentes de la
biblioteca:

 * Aplicación externa
 * Interfaz principal (`P2PSession` y `P2PSessionListener`)
 * Implementación del protocolo (protocolo binario y seguridad)
 * Plataforma (D-Bus y Android)
 * Utilidades

![Componentes](build/images/components.pdf)

\clearpage

# Diagramas de secuencia

A continuación se muestran los flujos operacionales principales usando
diagramas de secuencia.

![Descubrimiento de dispositivos](build/images/seq-peer-discovery.pdf)

![Establecimiento de conexión](build/images/seq-connection.pdf)

![Intercambio de claves y establecimiento de un canal seguro\label{fig:key-exchange}](build/images/seq-key-exchange.pdf)

![Intercambio de mensaje seguro](build/images/seq-message.pdf)
