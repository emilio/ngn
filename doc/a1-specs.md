\chapter{Especificaciones del sistema}
\label{annex:specs}

# Introducción

En este anexo se recoge la especificación de requisitos del Trabajo de Fin de Grado.

Para ello, el documento sigue, de manera aproximada, la estructura de documento
de requisitos de sistema propuesta por Durán Toro y Bernárdez Jiménez en su
*Metodología para la Elicitación de Requisitos de Sistemas Software*
\cite{req-duran-bernardez}.

También se utilizan las plantillas para especificación de requisitos presentados
por los mismos.

Los requisitos que se recogen en el presente anexo han sido obtenidos a partir
de entrevistas con los tutores del proyecto, Guillermo González Talaván y Pedro
Martín Vallejo Llamas. A partir de estas, se han elaborado tanto la propuesta,
que incluye los objetivos funcionales del sistema, como el resto de requisitos,
funcionales y no funcionales, recogidos en este anexo.

## Propuesta de Trabajo de Fin de Grado

La propuesta se coordinó con los tutores via gestión directa. Se incluyen a
continuación las partes relevantes de la propuesta, que se coordinó con los tutores por gesta:

### Descripción

El acceso a internet no es tan universal como suele parecer. Sin embargo,
dispositivos convencionales al alcance de la mayoría de la población soportan
comunicarse entre ellos de manera directa, via tecnologías estándar como
Bluetooth, WiFi-Direct, u otras.

Estas tecnologías tienen casos de usos muy variados, como comunicación en
situaciones de emergencia o lugares remotos, intercambio de datos de manera más
privada que una conexión a internet convencional...

A pesar de ello, su grado de adopción no es particularmente grande, en parte
por la dificultad de uso de estas tecnologías en comparación con internet. Se
desarrollará una biblioteca que abstraiga sobre diferentes tecnologías de
comunicación directa, y además proporcione capacidades de agrupación,
identificación, y opcionalmente enrutamiento, de más alto nivel.

### Objetivos funcionales

* La biblioteca permitirá a varios dispositivos enviar mensajes entre ellos sin
  necesidad de conexión a internet.
* La biblioteca proveerá una abstracción de bajo nivel sobre la tecnología
  física de comunicación.
* Tendrá al menos una implementación como prueba de concepto.
* Opcionalmente, la biblioteca también abstraerá diferencias entre plataformas
  / sistemas operativos.
* La biblioteca proveerá una abstracción de más alto nivel que permitirá:
  * Formación de grupos lógicos dentro de un grupo físico. Opcionalmente, se
    investigará la posibilidad de que un grupo lógico abarque más de un grupo
    físico.
  * Identificación (via sistema de clave pública / privada o similar),
    independiente de la capa física.
  * Opcionalmente, enrutado de mensajes via: Broadcast / Broadcast a un grupo
    lógico / Mensaje directo entre dos nodos lógicos (identidades).
* Se desarrollará una aplicación sencilla que demuestre las capacidades de la
  biblioteca.

### Entornos de desarrollo y explotación

Vim, Android, Android Studio, rr, Linux, Kotlin, Java, C, C++, Rust, Python.

# Participantes en el proyecto

En este proyecto hay tres participantes: El alumno y los dos tutores,
pertenecientes ambos a la misma organización, la Universidad de Salamanca.

Participante      Emilio Cobos Álvarez
----------------  ----------------------
Rol               Desarrollador
Es desarrollador  Sí
Es cliente        No
Es usuario        Sí
Comentarios       Ninguno

: Participante: Emilio Cobos Álvarez

Participante      Guillermo González Talaván
----------------  ------------------------
Rol               Tutor
Es desarrollador  No
Es cliente        Sí
Es usuario        Sí
Comentarios       Ninguno

: Participante: Guillermo González Talaván

Participante      Pedro Martín Vallejo Llamas
----------------  ------------------------
Rol               Tutor
Es desarrollador  No
Es cliente        No
Es usuario        Sí
Comentarios       Ninguno

: Participante: Pedro Martín Vallejo Llamas

Organización      Universidad de Salamanca
----------------  ----------------
Dirección         Patio de Escuelas Mayores, 1, 37008, Salamanca, España
Teléfono          +34 923 29 44 00
Fax               ---
Comentarios       Ninguno

: Organización: Universidad de Salamanca

# Objetivos del sistema

Se detallan a continuación los objetivos que se pretenden alcanzar con el
desarrollo del sistema.

OBJ-1        Abstracción de la capa física
-----        --------------
Versión      1.0 (20/10/2024)
Autores      Emilio Cobos Álvarez
Fuentes      Guillermo González Talaván, Pedro Martín Vallejo Llamas
Descripción  Diseñar e implementar un módulo que oculte las complejidades de
             las distintas tecnologías de comunicación directa (Bluetooth,
             WiFi-Direct, etc.), permitiendo enviar y recibir mensajes sin
             necesidad de conocer las APIs específicas.
Subobjetivos ---
Importancia  vital
Urgencia     inmediatamente
Estado       validado
Estabilidad  alta
Comentarios  ninguno

: OBJ-1: Abstracción de la capa física

OBJ-2        Interoperabilidad multiplataforma
-----        --------------
Versión      1.0 (20/10/2024)
Autores      Emilio Cobos Álvarez
Fuentes      Guillermo González Talaván, Pedro Martín Vallejo Llamas
Descripción  Diseñar abstracciones que permitan que la biblioteca funcione en
             múltiples sistemas operativos (Android, iOS, Linux, Windows),
             garantizando compatibilidad con la abstracción de capa física.
Subobjetivos ---
Importancia  vital
Urgencia     inmediatamente
Estado       validado
Estabilidad  alta
Comentarios  ninguno

: OBJ-2: Interoperabilidad multiplataforma

OBJ-3        Gestión de grupos lógicos
-----        --------------
Versión      1.0 (20/10/2024)
Autores      Emilio Cobos Álvarez
Fuentes      Guillermo González Talaván
Descripción  Implementar la capacidad de formar grupos lógicos dentro de una
             red física, explorar la posibilidad de interconectar varios grupos
             físicos.
Subobjetivos ---
Importancia  quedaría bien
Urgencia     puede esperar
Estado       validado
Estabilidad  alta
Comentarios  ninguno

: OBJ-3: Gestión de grupos lógicos

OBJ-4        Identificación y seguridad
-----        --------------
Versión      1.0 (20/10/2024)
Autores      Emilio Cobos Álvarez
Fuentes      Guillermo González Talaván, Pedro Martín Vallejo Llamas
Descripción  Incorporar un sistema de identificación basado en criptografía de
             clave pública/privada, garantizando autenticidad y privacidad de
             mensajes, independiente de la tecnología física de comunicación.
Subobjetivos ---
Importancia  vital
Urgencia     hay presión
Estado       validado
Estabilidad  alta
Comentarios  ninguno

: OBJ-4: Identificación y seguridad

OBJ-5        Prueba de concepto
-----        --------------
Versión      1.0 (20/10/2024)
Autores      Emilio Cobos Álvarez
Fuentes      Guillermo González Talaván, Pedro Martín Vallejo Llamas
Descripción  Implementar una aplicación sencilla como validación de la
             biblioteca y medio para obtener retroalimentación de facilidad de
             uso y estabilidad.
Subobjetivos ---
Importancia  vital
Urgencia     hay presión
Estado       validado
Estabilidad  alta
Comentarios  ninguno

: OBJ-5: Prueba de concepto

# Catálogo de requisitos del sistema

TODO

## Requisitos de la información

## Requisitos funcionales

## Requisitos no funcionales

# Matriz de rastreabilidad objetivo/requisitos
