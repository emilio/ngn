\chapter{Hipótesis, restricciones y alcance}

<!--
  La estructura de este apartado dependerá de la naturaleza del TFG. Al menos,
  se sugiere:
   * Detallar las restricciones del proyecto teniendo en cuenta los requisitos
     no funcionales, justificando su aparición.
   * Describir el impacto esperado con la propuesta, explicando las
     ventajas/mejoras que va a suponer el producto/servicio resultante del
     trabajo para el usuario, la sociedad o un sector empresarial, entre otras
     casuísticas.
-->

En este capítulo se detallan la hipótesis inicial que ha dado lugar al
proyecto, las restricciones que tiene la solución, y el impacto que se espera
de la misma.

# Hipótesis

Como se ha comentado en el \cref{chap:antecedentes}, la principal hipótesis es
que las tecnologías de conexión punto a punto no están tan extendidas como
cabría esperar, a pesar de sus características muy beneficiosas para el usuario
final (resiliencia, privacidad).

La principal hipótesis del trabajo es que una interfaz más sencilla de
utilizar, que abstraiga los detalles profundos del direccionamiento, podría
aumentar la adopción de estas tecnologías.

# Restricciones

En esta sección se detallan las limitaciones que han motivado las decisiones
tomadas durante el desarrollo del proyecto. Se han elaborado a partir de los
requisitos no funcionales, especificados en el \cref{sec:nfr}, y más en detalle
en el *Anexo I. Especificaciones del sistema*.

## Restricciones técnicas: entorno de explotación

La librería ha de ser multiplataforma, lo cual ha supuesto una serie de
restricciones técnicas, como el uso de lenguajes que soporten todas las
plataformas necesarias.

Similarmente, el requisito de que funcione en plataformas móviles fuerza a usar
determinadas APIs de esos sistemas operativos.

## Restricciones técnicas: Transporte inicial y limitaciones del sistema operativo

A la hora de realizar la elección de qué capa de transporte usar inicialmente
para esta librería, se eligió WiFi Direct por una variedad de razones:

 * Disponibilidad en Android \cite{wifi-direct-android} y Linux \cite{wifi-direct-linux}.
 * Mayor rango de alcance comparado con Bluetooth y Bluetooth LE \cite{wifi-direct-range}.
 * Soporte para varios grupos físicos en la misma tarjeta Wi-Fi
   \cite{wifi-direct-spec}, lo cual permite en teoría extender el alcance de la
   red infinitamente, dados los suficientes nodos intermedios.

Sin embargo, Android no permite a un mismo dispositivo estar conectado a dos
grupos de WiFi-Direct a la vez (a pesar de usar wpa_supplicant, que lo
soporta, y tener una interfaz interna,
[WifiP2pGroupList](https://cs.android.com/android/platform/superproject/main/+/main:packages/modules/Wifi/framework/java/android/net/wifi/p2p/WifiP2pGroupList.java;drc=9767925c3dbc08eeb6990a7e1109b916910b846c)),
aunque es un área activa de investigación \cite{wifi-direct-multigroup}.

Una línea de trabajo futura muy interesante que parece factible podría ser
expandir la red usando una variedad de transportes físicos (implementar
Bluetooth, y usar Bluetooth como conexión entre dos grupos).

## Restricciones técnicas: Acceso a identificadores

Otra restricción interesante que cambió el diseño de la interfaz es que Android
[restringe el acceso a la dirección MAC del
dispositivo](https://cs.android.com/android/platform/superproject/main/+/main:packages/modules/Wifi/service/java/com/android/server/wifi/p2p/WifiP2pServiceImpl.java;l=7502;drc=61197364367c9e404c7da6900658f1b16c42d0da)
con un permiso de sistema (que aplicaciones normales no pueden solicitar).

Similarmente, wpa_supplicant no soportaba exponer esta dirección directamente,
aunque se ha enviado un
[parche](https://lists.infradead.org/pipermail/hostap/2025-May/043428.html) y
tests para hacerlo.

Esta ID sería útil, porque es la que otros dispositivos y la capa de transporte
ven, pero como compromiso, la librería soporta asociarse por nombre
inicialmente (aunque eso por supuesto tiene más posibilidades de colisiones).

## Restricciones técnicas: Permisos en Linux

El ecosistema de Linux es muy variado, y no se ha hecho un estudio exhaustivo
sobre qué distribuciones limitan el acceso por \Gls{D-Bus} a `wpa_supplicant`,
pero al menos Arch Linux limita el acceso a `root` por defecto:

```xml
<!DOCTYPE busconfig PUBLIC
 "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
        <policy user="root">
                <allow own="fi.w1.wpa_supplicant1"/>

                <allow send_destination="fi.w1.wpa_supplicant1"/>
                <allow send_interface="fi.w1.wpa_supplicant1"/>
                <allow receive_sender="fi.w1.wpa_supplicant1" receive_type="signal"/>
        </policy>
        <policy context="default">
                <deny own="fi.w1.wpa_supplicant1"/>
                <deny send_destination="fi.w1.wpa_supplicant1"/>
                <deny receive_sender="fi.w1.wpa_supplicant1" receive_type="signal"/>
        </policy>
</busconfig>
```

Por lo tanto para acceder a esas APIs desde user-space se ha tenido que añadir
algo como:

```xml
<policy group="wheel">
        <allow own="fi.w1.wpa_supplicant1"/>

        <allow send_destination="fi.w1.wpa_supplicant1"/>
        <allow send_interface="fi.w1.wpa_supplicant1"/>
        <allow receive_sender="fi.w1.wpa_supplicant1" receive_type="signal"/>
</policy>
```

Para permitir el acceso a todos los usuarios del grupo `wheel`. Otras
alternativas serían usar un `dbus-daemon` diferente, como se ha hecho para
testear localmente.

## Restricciones técnicas: DHCP 

TODO

## Restricciones de usabilidad: Permisos en Android

Usar WiFi Direct en Android [requiere amplios
permisos](https://developer.android.com/develop/connectivity/wifi/wifi-direct#permissions),
y tener los servicios de ubicación activados.

Esto fuerza a la biblioteca a depender más del contexto de la aplicación en
Android, y a la inicialización a ser asíncrona, ya que estos permisos requieren
interacción del usuario.

La hipótesis para requerir esto es que técnicamente puedes usar el escaneo de
redes WiFi para geolocalización, usando bases de datos como
[BeaconDB](https://beacondb.net/).

## Restricciones de usabilidad: Interacción del usuario

Idealmente, para casos de uso como crear redes ad-hoc, la biblioteca o
aplicación que la use podría anunciarse y conectarse a dispositivos sin
interacción.

Sin embargo Android no soporta ese caso de uso, y requiere una interacción la
primera vez que intentas conectarte a un dispositivo. Por lo tanto la
información que la librería puede exponer inicialmente sobre el dispositivo es
mucho más limitada.

## Restricciones de usabilidad: Interacción entre `wpa_supplicant` y `NetworkManager`

TODO

# Alcance funcional del proyecto

