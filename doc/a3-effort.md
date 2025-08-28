\chapter{Estimación del tamaño y esfuerzo}

En el presente anexo se recoge la metodología de trabajo empleada en el
desarrollo del proyecto, la planificación temporal de las tareas realizadas,
las métricas empleadas para la planificación y sus estimaciones.

El proyecto se desarrollará siguiendo la metodología de trabajo ágil Scrum (ver
\cref{subsec:scrum}), una metodología que permite una planificación iterativa
e incremental, facilitando la adaptación a los cambios y la mejora continua del
proceso de desarrollo.

Esta metodología se adapta bien al proyecto, ya que permite que tanto el autor
como los tutores puedan llevar un seguimiento del avance del proyecto, y
modificar las tareas a realizar en función de los resultados obtenidos en cada
iteración.

Por ejemplo, si una tarea resulta más compleja de lo previsto inicialmente, se
podrá decidir si merece la pena dedicar más esfuerzo a esa tarea, o si es mejor
sustituirla por una alternativa más fácil de implementar, o incluso eliminarla
si no es esencial para el proyecto.

También evita la necesidad de una planificación detallada y exhaustiva al
inicio del proyecto, que hubiera sido imposible dada la variabilidad de la
cantidad de tiempo que el autor ha podido dedicar al proyecto (por su trabajo a
tiempo completo y cargas familiares), y la gran cantidad de incógnitas que
existían al comienzo del mismo.

# Scrum

Este proyecto se ha desarrollado siguiendo el marco de trabajo Scrum[^1]. A
continuación se presenta una breve introducción a Scrum, basada parcialmente en
la Guía de Scrum\cite{scrum-guide}, documento que define el marco de
trabajo Scrum y que se actualiza periódicamente.

También se explica, para cada una de las secciones, cómo se aplica esta parte
del marco teórico al desarrollo del proyecto, en la práctica.

![Diagrama gráfico del proceso Scrum (Fuente: Lakeworks)](images/scrum.pdf)

[^1]: Aunque lo parezca, Scrum no es un acrónimo, sino un término tomado del
rugby, que hace referencia a una formación de jugadores en el campo, en la que
cada uno de los jugadores tiene un rol específico, y todos trabajan juntos para
conseguir un objetivo común (el balón) \cite{scrum-meaning}

## Definición

Según los autores originales de Scrum, Schwaber y Sutherland \cite{scrum-guide},
\enquote{Scrum es un marco [de trabajo] ligero que ayuda a las personas,
equipos y organizaciones a generar valor a través de soluciones adaptables para
problemas complejos} (p. 3).

El enfoque del proceso Scrum es «iterativo e incremental», lo que significa que
se realizan entregas periódicas de un producto que va evolucionando a lo largo
del tiempo, y que se va adaptando a las necesidades del cliente.

## Roles

En Scrum, existen tres roles principales: el *Product Owner*, el *Scrum Master*
y el *Development Team*.

### Product Owner

El *Product Owner* o dueño del producto es el responsable de maximizar el valor
del producto resultante del trabajo del equipo Scrum. El dueño del producto es
quien define el objetivo del producto, y quien gestiona el *Product Backlog*.

### Scrum Master

El *Scrum Master* es el responsable de asegurar que Scrum se entiende y se
aplica correctamente. Actúa de intermediario entre el *Product Owner* y el
*Development Team*, facilitando la comunicación y la colaboración entre ambos,
y ayudando a resolver los obstáculos que puedan encontrar durante el proceso de
desarrollo.

Además, al delegársele las labores de seguimiento del flujo de trabajo, permite
que el equipo de desarrollo se centre en progresar hacia el objetivo del
producto.

### Development Team

El *Development Team* o equipo de desarrollo es el grupo de personas que se
encarga de desarrollar las funcionalidades de los incrementos del producto.

El equipo de desarrollo es autoorganizado, lo que significa que son ellos
quienes deciden cómo llevar a cabo el trabajo, y son responsables de la calidad
del producto que entregan. \cite{scrum-guide} (p. 7).

El equipo de desarrollo es multidisciplinar, y puede estar formado por
desarrolladores, diseñadores, testers, etc. El equipo tiene todas las
habilidades necesarias para generar los incrementos del producto.

Los miembros del equipo de desarrollo son iguales entre sí, y no hay jerarquías
ni equipos de trabajo especializados.

El tamaño del equipo de desarrollo debe ser lo suficientemente grande como para
completar el trabajo, pero lo suficientemente pequeño como para que la
comunicación y la colaboración sean efectivas. La Guía de Scrum recomienda un
tamaño de entre 3 y 9 personas \cite{scrum-guide} (p. 7)

## Aplicación al Trabajo de Fin de Grado

En el caso del Trabajo de Fin de Grado, los *Product Owners* son los tutores
del TFG, quienes definen el objetivo del producto y gestiona la pila
del producto.

El autor cumple los roles tanto de Scrum Master como el equipo de desarrollo,
asegurándose de cumplir con el marco de trabajo Scrum, hablando con los
*Product Owners* para definir el objetivo del producto y la pila del producto,
y desarrollando los incrementos del producto.

## Flujo de trabajo

El trabajo en Scrum se organiza en sprints, que son «eventos de longitud fija de
un mes o menos» \cite{scrum-guide} (p. 8), cuyo objetivo es entregar uno (o
más) incrementos del producto. Al finalizar un sprint, se comienza el
siguiente, volviendo a tener lugar todas las reuniones y eventos necesarios, y
así sucesivamente hasta que se alcance el objetivo del producto.

### Sprint Planning

Cada sprint comienza con una reunión de planificación, llamada *Sprint
Planning*, en la que los integrantes del equipo definen el trabajo a realizar
durante el sprint (confeccionando el *Sprint Backlog*) y establecen un objetivo
para el mismo.

### Daily Scrum

Tras la planificación, el equipo trabaja en las tareas definidas, y se reúne
diariamente en una reunión llamada *Daily Scrum*, cuyo propósito es
\enquote{inspeccionar el progreso hacia el Objetivo del Sprint y adaptar el
Sprint Backlog según sea necesario, ajustando el próximo trabajo planeado}
\cite{scrum-guide} (p. 9).

El *Daily Scrum* es una reunión breve, de máximo 15 minutos \cite{scrum-guide}
(p. 9), en la que los desarrolladores comunican su progreso, los obstáculos que
han encontrado y lo que planean hacer a continuación.

### Sprint Review

Durante la Sprint Review (en español, revisión del sprint), que ocurre al final
de cada sprint, como penúltimo evento, participan todos los roles de Scrum, y
se revisa el trabajo realizado durante el sprint, el equipo presenta el
incremento del producto.

Entonces se recoge la retroalimentación del dueño del producto o Product Owner,
y se discute el progreso hacia el objetivo del producto y qué hacer a continuación.

### Sprint Retrospective

Finalmente, tras la revisión del sprint, se realiza una reunión retrospectiva
del sprint o Sprint Retrospective, en la que el equipo reflexiona sobre el
trabajo realizado, y sobre qué cambios hacer para mejorar su eficacia y
eficiencia de cara al siguiente sprint.

En cada sprint se planifica el trabajo a realizar (*Sprint Planning*), se
trabaja y revisa el trabajo realizado (*Daily Scrum*) y, al final (pero aún
dentro del sprint), se revisa el trabajo realizado (*Sprint Review*) y se
reflexiona sobre el trabajo realizado y cómo mejorar para el siguiente sprint
(*Sprint Retrospective*).

### Aplicación al Trabajo de Fin de Grado

El caso del Trabajo de Fin de Grado es un caso particular, ya que el equipo de
desarrollo está formado por una sola persona, el autor. Por esto, no se han
realizado reuniones diarias (*Daily Scrum*).

Las reuniones de revisión del sprint (Sprint Review) se han llevado a cabo
semanalmente, y han consistido en una revisión del trabajo realizado durante la
semana, así como una revisión del progreso hacia el objetivo del producto, y
una discusión sobre qué hacer a continuación.

## Artefactos

Los artefactos de Scrum son los elementos que se utilizan para gestionar el
trabajo y el progreso del equipo, y son tres: el *Product Backlog*, el *Sprint
Backlog* y el *Incremento*. Cada uno de ellos se utiliza para conseguir llegar
a una meta específica, y se actualizan periódicamente a lo largo del proceso de
desarrollo.

### Product Backlog o pila del producto

La pila del producto \enquote{es una lista emergente y ordenada de lo que se
necesita para mejorar el producto} \cite{scrum-guide} (p. 11). Las tareas que
se incluyen en la pila del producto son las que, durante la planificación del
sprint, se seleccionan para formar parte del *Sprint Backlog*.

El objetivo del producto es un objetivo a largo plazo sobre el que se puede
planificar qué tareas incluir en la pila del producto. Surge de la interacción
con el dueño del producto o *Product Owner*.

### Sprint Backlog o pila del sprint

La pila del sprint \enquote{se compone del objetivo sprint (por qué), el
conjunto de elementos de trabajo pendiente de producto seleccionados para el
Sprint (qué), así como un plan accionable para entregar el incremento (cómo)}
\cite{scrum-guide} (p. 12).

Es decir, las tareas que se van a realizar durante el sprint, y que se
seleccionan de la pila del producto durante la planificación del sprint. La
desarrollan los desarrolladores, y se revisa (y reforma si fuese necesario)
durante el *Daily Scrum*.

### Increment o incremento del producto

Los incrementos son cada uno de los pasos (incrementales) que se dan para
alcanzar el objetivo del producto. Son \enquote{aditivo[s] a todos los incrementos
anteriores y verificado[s] a fondo, asegurando que todos los incrementos
funcionen juntos} \cite{scrum-guide} (p. 12).

Son, además, utilizables, lo que significa que se pueden entregar al cliente, y
que este puede utilizarlos para obtener valor del producto. Durante cada
sprint, como se ha visto anteriormente, se pueden entregar uno o varios
incrementos del producto.

### Aplicación al Trabajo de Fin de Grado

Los artefactos Scrum se han mantenido intactos al aplicar el marco teórico al
TFG, aunque no se ha mantenido un backlog continuamente durante la duración del
proyecto, ya que sólo hay un desarrollador y generalmente se han planeado los
sprints continuamente en base a la anterior.

Una planificación y estimación más detallada por adelantado hubiera sido tal
vez beneficiosa pero, dada la naturaleza de investigación especialmente durante
los primeros meses del proyecto, hubiera sido de utilidad limitada.

## Estimaciones de tiempo

Antes de comenzar los sprint, el equipo de desarrollo debe estimar el tiempo
que tardará en completar cada tarea de la pila del producto. Esto se puede
hacer calculando el tiempo en horas que se tardará, directamente, o bien
mediante unidades de medida relativas, como puntos de historia (*story points*)
o tamaños de camiseta (*t-shirt sizes*).

Comenzando por los *story points*, según escribe Derek Davidson
\cite{why-story-points}, \enquote{un punto de historia es una unidad de medida
relativa, decidida y utilizada por equipos Scrum individuales, para
proporcionar estimaciones relativas del esfuerzo necesario para completar los
requisitos}.

El uso de puntos de historia es una práctica común en Scrum, y permite estimar
el esfuerzo necesario para completar una tarea de forma relativa, comparándola
con otras tareas, en lugar de absoluta.

La ventaja de utilizar puntos de historia en vez de una estimación clásica de
tiempo es que, según el coautor de la Guía de Scrum, Sutherland
\cite{story-points-vs-hours} \enquote{los puntos de historia dan estimaciones
más precisas, reducen el tiempo de planificación drásticamente, predicen las
fechas de lanzamiento con mayor precisión, y ayudan a
los equipos a mejorar su desempeño}.

Los tamaños de camiseta son otra forma de estimar el esfuerzo necesario para
completar una tarea. Son muy parecidos a los puntos de historia, pero en lugar
de utilizar una escala numérica, se utilizan tamaños de camiseta (XS, S, M, L,
XL, XXL) para representar el esfuerzo necesario para completar la tarea.

Esta forma de estimación es más intuitiva, y puede ser más fácil de entender
para personas que no están familiarizadas con Scrum o con la estimación de
tiempo en general. Sin embargo, puede ser menos precisa que los puntos de
historia, ya que no se puede comparar directamente el esfuerzo necesario para
completar una tarea con el de otra.

Además, resultan más complicados de utilizar a la hora de desarrollar gráficos,
ya que no se pueden utilizar directamente para calcular el trabajo pendiente o
el trabajo realizado, y requieren una conversión a puntos de historia para este
fin.

### Aplicación al Trabajo de Fin de Grado

Para el Trabajo de Fin de Grado, se han utilizado *t-shirt sizes* como medida
de estimación de esfuerzo, y no se ha realizado una contabilización exhaustiva
del tiempo empleado en cada tarea.

Esto se debe a que se considera que lo importante es el esfuerzo necesario para
completar una tarea, y no el tiempo que se tarda en completarla, siendo el
tiempo una medida complicada de estimar, y que puede variar mucho, ya no solo
dependiendo del esfuerzo invertido, sino también de factores externos como el
el tiempo disponible para trabajar en la tarea.

# Planificación temporal

En el presente capítulo se detallará la planificación temporal del proyecto,
así como las métricas que se han utilizado para su elaboración.

Dadas las restricciones temporales, la planificación temporal ha sido bastante
dinámica. Las reuniones de *Sprint Planning* se utilizaron para decidir las
tareas a completar durante el sprint, y estimar su esfuerzo, generalmente
tratando de dividir las tareas de tal manera que pudieran ser ejecutadas
durante el *sprint* en la medida de lo posible.

A continuación se presenta un resumen de alto nivel de las fases del
proyecto, seguido de un resumen sprint a sprint.

## Resumen general

El proyecto se ha desarrollado de manera relativamente consistente desde
Octubre de 2024 a Agosto de 2025, con algunos parones intermitentes
(generalmente por la amplia carga de trabajo del autor), especialmente en
Abril de 2025.

### Exploración inicial (Octubre de 2024)

Se realizó una exploración inicial de las capacidades de comunicación
peer-to-peer de Android y Linux.

Se eligió tentativamente WiFi Direct como capa de transporte inicial, y se
desarrolló un [prototipo](https://github.com/emilio/android-wifip2p-test) de
aplicación para Android, tras una primera familiarización con Android Studio.

Se determinó que para el primer prototipo se usaría Linux como plataforma
soportada (ver \cref{subsec:testing} y \cref{subsec:transport-impl}).

### Desarrollo inicial en Linux (Diciembre de 2024 a Enero de 2025)

Se realizaron los bloques iniciales usando D-Bus para comunicarse con
wpa_supplicant. Una buena parte de esta fase fue diagnosticar problemas con los
que no se contaron inicialmente (ver \cref{subsec:transport-impl} y
\cref{sec:restrictions}).

### Desarrollo del protocolo principal (Febrero a Mayo de 2025)

Se consiguió un entorno de pruebas estable, se desarrolló todo el protocolo
binario y un intercambio de mensajes básico usando TCP.

Se desarrolló también la implementación inicial de una interfaz de usuario para
pruebas usando GTK.

Se determinaron las abstracciones principales, y se refactorizó la
implementación de Linux con el objetivo de introducir soporte para Android a
continuación.

### Integración con Android (Junio a Julio de 2025)

Se desarrolló una aplicación de pruebas básica para Android, junto a toda
la implementación de la librería y la integración con Java usando la \gls{JNI}.

También se hicieron ajustes al protocolo para acomodar las limitaciones /
restricciones en Android.

Finalmente, se ajustó la aplicación de pruebas para convertirla en la
aplicación de demostración (el juego de 2048).

### Criptografía y seguridad (Julio a Agosto de 2025)

Se implementó la identificación de clave pública y firma de mensajes
inicialmente, y posteriormente se realizaron los ajustes necesarios al
protocolo para realizar el intercambio de claves y el paso de mensajes cifrado.

### Finalización de la documentación (Agosto de 2025)

Se habían hecho varias partes de la documentación como tareas auxiliares en
otros sprints, y se habían documentado las cosas importantes de manera no
estructurada en un fichero de texto en el repositorio, pero fue en Agosto
cuando se formalizó toda la documentación.

## Desglose detallado

A continuación se detallan los sprints donde hubo actividad. Generalmente la
actividad puede ser comprobada via el log del sistema de control de versiones.

### Sprint 1: Del 1 al 6 de Octubre de 2024

Se investigaron las diferentes alternativas para implementar la capa de
transporte. Se eligió probar WiFi Direct como capa de transporte para el prototipo por las razones detalladas en el \cref{subsec:transport-decision}.

### Sprint 2: Del 7 al 13 de Octubre de 2024

Se desarrolló un prototipo de aplicación en Android que conectara dos
dispositivos y enviara un mensaje entre ellos.

Tras la reunión de revisión del sprint se decidió afrontar la implementación
inicial en Linux.

### Sprint 3: Del 15 al 21 de Diciembre de 2024

Se creó el repositorio principal (independiente al prototipo). Se investigó el
estado de la comunicación via D-Bus en Linux, y se crearon las primeras
interfaces usando zbus, consiguiendo un listado de dispositivos adyacentes.

Las pruebas iniciales revelaron problemas inesperados (descritos en el
\cref{sec:restrictions}). Se encontraron problemas con zbus y se propusieron
algunas soluciones (ver \cref{subsec:external-contributions}).

### Sprint 4: Del 30 de Diciembre de 2024 al 5 de Enero de 2025

Este sprint por razones obvias (navidad) no hubo reunión de *Sprint Planning*.
Se depuraron los problemas de comunicación con D-Bus.

### Sprint 5: Del 6 al 12 de Enero de 2025

Se hizo mucho progreso en la depuración, y se consiguió conectar el ordenador
de sobremesa del autor con su portátil (ambos corriendo Linux), descubriendo
problemas nuevos, como que WiFi direct en sí mismo no proporciona la dirección
IP de los dispositivos, por lo que tiene que ser derivada de alguna otra
manera. Ver \cref{subsec:dhcp} para algunos de los detalles al respecto.

### Sprint 6: Del 20 al 26 de Enero de 2025

Se mejoró la demo de DBus para soportar reintento de conexiones. También se
depuró wpa_supplicant para intentar entender algunos de los problemas descritos
anteriormente.

### Sprint 7: Del 10 al 16 de Febrero de 2025

Se consiguió establecer una comunicación UDP entre dispositivos via direcciones
de link local \cite{rfc4862} (sección 5.3).

También se creó la infraestructura para la documentación del proyecto (portada
del proyecto, `Makefile`, secciones iniciales con \LaTeX...).

### Sprint 8: Del 17 al 23 de Febrero de 2025

Se hizo progreso en poder probar la capa de transporte. Se descubrió la razón
por la que el código funcionaba en uno de los ordenadores del autor pero no en
el otro (ver \cref{subsec:dhcp}).

Se empezó a estructurar el código para prepararlo para ser reutilizado.

### Sprint 9: Del 24 de Febrero al 2 de Marzo de 2025

Se avanzó en la documentación, creando el glosario y compartiendo la estructura
con otros alumnos.

### Sprint 10: Del 3 al 9 de Marzo de 2025

Se creó la infraestructura para la bibliografía de la memoria, y se hizo algo
de progreso al respecto.

Se dividió la estructura del código en una biblioteca y una aplicación de
ejemplo.

### Sprint 11: Del 24 al 30 de Marzo de 2025

Se implementó el protocolo binario de paso de mensajes y se implementaron
notificaciones para el manejo del grupo en la librería.

### Sprint 12: Del 1 al 7 de Abril de 2025

Se elaboró una interfaz de paso de mensajes independiente a la capa de
transporte.

### Sprint 13: Del 5 al 11 de Mayo de 2025

Se depuró un error de memoria y se mejoró la infraestructura de pruebas para
integrar rr (ver \cref{subsec:rr}) y \Gls{ASan}.

### Sprint 14: Del 12 al 18 de Mayo de 2025

Se implementó la gestión básica de grupos, y se implementó funcionalidad para
auto-unirse a un grupo en wpa_supplicant (ver
\cref{subsec:external-contributions}).

### Sprint 15: Del 26 de Mayo al 1 de Junio de 2025

Se implementó una interfaz básica de pruebas usando GTK, para poder probar
situaciones con más de dos dispositivos por grupo físico más fácilmente.

### Sprint 16: Del 9 al 15 de Junio 2025

Se creó la aplicación básica de Android con un esqueleto básico del backend
para Android.

### Sprint 17: Del 16 al 22 de Junio 2025

Se integró mucho más profundamente la librería con Java, incluyendo poder
incluir y llamar al código de Rust desde la aplicación. La biblioteca aún no
era funcional.

### Sprint 18: Del 23 al 29 de Junio 2025

Se consiguió una integración básica en Android y se creó infraestructura para
poder testear Android con un sólo teléfono (comunicando el dispositivo Android
con Linux) para poder hacer progreso sin tener dos dispositivos.

Se ajustó el descubrimiento de direcciones para soportar IPv4 también, ya que
Android usa DHCP + IPv4 por defecto.

### Sprint 19: Del 7 al 13 de Julio 2025

Se hizo algo de progreso en tener la librería funcional en Android, arreglando
la gestión de grupos.

### Sprint 20: Del 14 al 20 de Julio 2025

Se implementó la identidad y firma de mensajes, y se investigó el cifrado de
mensajes también. Se implementaron varias APIs de Android también.

### Sprint 21: Del 21 al 27 de Julio 2025

Se implementaron muchas mejoras de interfaz en Android y se implementó el juego
2048, que se demostró a los tutores en la retrospectiva del *sprint*.

### Sprint 22: Del 5 al 11 de Agosto de 2025

Se empezó con la redacción más completa de la memoria, integrando también
varias mejoras (soporte para código, etc) a la misma.

### Sprint 23: Del 12 al 18 de Agosto de 2025

Se implementó y documentó el cifrado de mensajes punto a punto y el intercambio
de claves. Se envió a `wpa_supplicant` una mejora necesaria para ello (ver
\cref{subsec:external-contributions}), y se verificó que la aplicación de
Android funcionaba con mensajes cifrados.

Todo el tiempo desde entonces se ha empleado en la documentación del trabajo.

# Conclusión

El uso de la metodología inspirada en Scrum en este proyecto ha permitido una
planificación y desarrollo ágil del mismo, con una buena adaptación a los
cambios y una mejora continua del producto.

No hay ninguna duda de que el uso de Scrum ha sido una buena elección para el
proyecto. Las frecuentes reuniones de revisión de los sprints (semanales) han
permitido una buena comunicación con los tutores y atajar muchos problemas
antes de que se convirtieran en grandes obstáculos.

En la práctica, siendo un desarrollo individual, se han podido tomar atajos que
en una aplicación más estricta de la metodología no sería posible (como
saltarse sprints, o replanificar a mitad de sprint). Algunos de estos atajos
recuerdan más a Kanban \cite{kanban} que a Scrum, más orientado a un flujo de
trabajo continuo.

El autor usa metodologías ágiles en su trabajo, y considera que usar una
versión más estricta del Scrum como se podía haber hecho (requiriendo
estructurar mucho más el backlog, anotar cada tarea con estimaciones, no
replanificar a mitad de *sprint*) hubiera sido un error, especialmente dadas
las restricciones temporales. Usar metodologías no ágiles tampoco hubiera sido
factible, por las mismas razones.
