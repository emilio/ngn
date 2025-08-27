\chapter{Plan de seguridad}

La biblioteca ha sido diseñada para proveer seguridad *en profundidad*. Ya se
han tocado muchos de estos detalles en la memoria (ver \cref{sec:crypto},
\cref{fig:key-exchange}). Este anexo servirá como repaso general a los varios
mecanismos de seguridad que se implementan para garantizar tanto la integridad
como la privacidad de los mensajes.

# Seguridad de la capa de transporte

La librería intencionadamente (ver \cref{obj-4}, ) *no asume que la capa de
transporte sea segura*. Esto quiere decir que si remplazáramos WiFi Direct (la
capa de transporte del prototipo) por una radio no cifrada, las propiedades de
seguridad de la librería serían las mismas.

Dicho eso, existen medidas de seguridad a la hora de usar WiFi Direct en
particular que tal vez sean dignas de mención.

Linux restringe el acceso a la gestión de redes P2P (ver
\cref{subsec:linux-permissions}), y Android provee requiere de interacción para
crear usuarios (ver \cref{fig:conn-request}).

WiFi-Direct generalmente utiliza \gls{WPS} como medida de seguridad para
prevenir que cualquier miembro externo se una: Es la aplicación o el sistema
operativo el que tiene que aceptar la conexión.

# Autenticidad de los mensajes

La autenticidad de los mensajes se garantiza con el firmado de los mismos
usando la clave de identidad de la sesión (que es una clave
[ed25519](https://ed25519.cr.yp.to/)).

Como se ha discutido anteriormente (ver \cref{sec:crypto}), esta clave no se
presume efímera.

# Privacidad de los mensajes

La privacidad de los mensajes se garantiza con el cifrado
\Gls{AES}-256-\Gls{GCM}, usando una clave de cifrado efímera para cada par de
nodos que se comunican entre sí. Esta clave efímera se genera usando X25519
\cite{rfc7748}, una método de \Gls{ECDH}.

Adicionalmente, se usa un contador como protección ante ataques de reproducción
(este es el `ring::aead::NonceSequence`). Sin embargo, no estoy convencido de
que merezca la pena en la práctica y podría ser re-evaluado en un futuro ya
que, al menos con la implementación actual, puede hacer que paquetes perdidos
acaben rompiendo la comunicación, efectivamente.

# Integridad de los mensajes

La integridad de los mensajes se garantiza por partida doble. Primero, con la
susodicha firma. Segundo, con el cifrado \Gls{AES}-256-\Gls{GCM}.
