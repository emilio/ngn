\chapter{Descripción de la solución propuesta}

<!--
  Se debe describir técnicamente el producto final con funcionalidades,
  capturas de pantalla, diagramas de flujo, etc.

  En este apartado se incluirá algún modo de probar el desarrollo realizado,
  bien mediante un enlace a la aplicación desplegada en algún servidor (si
  aplica al tipo de proyecto), un enlace al ejecutable o instalable (.exe,
  .apk, etc.), o si no aplica ninguna de estas, un vídeo demostrativo.
-->

En esta sección se presenta el software resultante del proyecto, se describirá
su funcionalidad, y se proveerán ejemplos de uso. Se proporcionará acceso a una
demo para Android.

# Acceso

Siendo una biblioteca de software, el producto principal está en el
repositorio, accesible tanto en [GitHub](https://github.com/emilio/ngn), como
en el [mirror personal](https://crisal.io/git/?p=ngn.git;a=summary).

La demo de Android se puede compilar con Android Studio, o descargar desde
[GitHub Releases](https://github.com/emilio/ngn/releases).

# Estructura general del proyecto

A continuación se expone una visión simplificada de la estructura del proyecto:

```
├── Cargo.toml
├── doc
├── examples
│   ├── android
│   └── dbus
├── LICENSE
├── README.md
├── src
│   ├── lib.rs
│   ├── platform
│   │   ├── android
│   │   │   ├── mod.rs
│   │   │   └── src/main/java/io/crisal/ngn
│   │   │       ├── NgnListener.kt
│   │   │       └── NgnSessionProxy.java
│   │   ├── dbus
│   │   │   ├── mod.rs
│   │   │   ├── store.rs
│   │   │   └── wpa_supplicant
│   │   └── mod.rs
│   ├── protocol
│   │   ├── encryption.rs
│   │   ├── identity.rs
│   │   ├── key_exchange.rs
│   │   ├── mod.rs
│   │   └── signing.rs
│   └── utils.rs
└── test
    ├── dbus-system-bus-mock.conf
    ├── setup-android.sh
    ├── setup.sh
    └── simple.conf
```

Todo el proyecto es parte del mismo paquete de `cargo`, definido en
`Cargo.toml`. Ahí es donde los datos básicos y dependencias están declaradas:

```toml
[package]
name = "ngn"
version = "0.1.0"
edition = "..."
license = "..."
# ...

[lib]
name = "ngn"
crate-type = ["cdylib", "lib"]

[dependencies]
tokio = { version = "1", features = ["full"] }
# ...
[target.'cfg(target_os = "android")'.dependencies]
jni = "0.21"
# ...
```

También donde se declaran la estructura y dependencias de la demo de Linux, que
vive en `examples/dbus`:

```toml
[dev-dependencies]
gtk = { version = "0.9.6", package = "gtk4", features = ["v4_18"] }
adw = { version = "0.7.2", package = "libadwaita", features = ["v1_4"] }

[[example]]
name = "dbus"
crate-type = ["bin"]
```

El código de Android también se divide en dos. La librería, en
`src/platform/android`, con su parte de Java / Kotlin en
`src/platform/android/src/main/java/io/crisal/ngn`, y la aplicación de
demostración en `examples/android`.

Por conveniencia, se ha usado [`tokio`](https://tokio.rs/) como
*[runtime](https://www.ncameron.org/blog/what-is-an-async-runtime/)* asíncrona.
El uso de tokio en la librería no es particularmente especial y se podrían
soportar varias *runtimes* sin problema.

# Interfaces y estructuras principales

## `P2PSession` y `P2PSessionListener`

La interfaz principal de la librería está en `src/lib.rs`, donde se define el
\gls{trait} `P2PSession`, cuya implementación varía por plataforma, y es la que
expone métodos para iniciar el descubrimiento de dispositivos
(`discover_peers`), conectarse (`connect_to_peer`) y enviar mensajes
(`message_peer`):

```rust
#[async_trait::async_trait]
pub trait P2PSession: Sized + Debug + Send + Sync + 'static {
    async fn new(
        args: Self::InitArgs<'_>,
        listener: Arc<dyn P2PSessionListener<Self>>,
    ) -> GenericResult<Arc<Self>>;
    async fn stop(&self) -> GenericResult<()>;
    async fn wait(&self) -> GenericResult<()>;
    async fn discover_peers(&self) -> GenericResult<()>;
    fn peer_identity(&self, id: PeerId) -> Option<protocol::PeerIdentity>;
    fn all_peers(&self) -> Vec<(PeerId, protocol::PeerIdentity)>;
    fn own_identity(&self) -> &protocol::identity::OwnIdentity;
    async fn connect_to_peer(&self, id: PeerId) -> GenericResult<()>;
    async fn message_peer(&self, id: PeerId, message: &[u8]) -> GenericResult<()>;
}
```

La inicialización de la sesión requiere un `P2PSessionListener`, que es la
forma de reaccionar a cambios de manera asíncrona. La implementación por
defecto simplemente loguea los eventos.

```rust
pub trait P2PSessionListener<S: P2PSession>: Debug + Send + Sync {
    fn peer_discovered(&self, _: &S, peer_id: PeerId);
    fn peer_lost(&self, _: &S, peer_id: PeerId);
    fn peer_discovery_stopped(&self, _: &S);
    fn joined_group(&self, _: &S, group_id: GroupId, is_go: bool);
    fn left_group(&self, _: &S, group_id: GroupId, is_go: bool);
    fn peer_joined_group(&self, _: &S, group_id: GroupId, peer_id: PeerId);
    fn peer_left_group(&self, _: &S, group_id: GroupId, peer_id: PeerId);
    fn peer_messaged(&self, _: &S, peer_id: PeerId, group_id: GroupId, message: &[u8]);
}
```

## Identificación de dispositivos y grupos

Los identificadores que se usan para el enrutamiento de mensajes (`PeerId` y
`GroupId`) son independientes de la capa de transporte y plataforma. Son
simplemente un *handle* de 64 bits:

```rust
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct PeerId(pub(crate) handy::Handle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct GroupId(pub(crate) handy::Handle);
```

El enrutamiento de mensajes a un `PeerId` es independiente del grupo físico
(`GroupId`) al que está conectado. El grupo físico se expone ahora mismo en el
`Listener`, pero es probable que se elimine porque no es necesario y es
probable que en otras capas de transporte no haya múltiples grupos.

La identidad *lógica* de un dispositivo (independiente de la capa de
transporte) es simplemente un *nick* (nombre) y una clave criptográfica:

```rust
#[derive(Debug)]
pub struct OwnIdentity {
    pub nickname: String,
    pub key_pair: KeyPair,
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct LogicalPeerIdentity {
    pub nickname: String,
    pub key: MaybeInvalidPublicKey,
}
```

# Enrutado de mensajes en WiFi Direct

Cuando se forma un grupo de WiFi Direct, hay efectivamente dos modos de
operación. En cada grupo, hay un GO o *Group Owner*, y el resto de miembros son
clientes.

Los clientes se pueden comunicar entre sí sin pasar por el GO **una vez sepan su
dirección IP**, pero es el GO el que tiene que encargarse de comunicar la
existencia de nuevos miembros a los existentes.

Durante la formación del grupo, la única dirección IP que podemos saber por
adelantado en todas las plataformas es la dirección del GO. En el mundo ideal,
podríamos depender the las direcciones de link local \cite{rfc4862}, para
obtener una dirección dada la dirección MAC de la interfaz, pero Android no la
expone, y aunque puedes [solicitar el uso de
IPv6](https://developer.android.com/reference/android/net/wifi/p2p/WifiP2pConfig#GROUP_CLIENT_IP_PROVISIONING_MODE_IPV6_LINK_LOCAL),
es una API bastante reciente.

Por lo tanto la solución que se adoptó establece **dos canales**, un canal de
*control* no encriptado, para la gestión del grupo e intercambio de claves,
y uno para la comunicación cifrada y firmada de mensajes.

El único puerto que tiene que ser conocido de antemano es el del canal de
control del *Group Owner*, el cual está definido en `src/protocol/mod.rs`:

```rust
/// The port the GO of the group listens to.
pub const GO_CONTROL_PORT: u16 = 9001;
```

El resto de puertos son dinámicos.

![Flujo de control al conectarse a un nuevo grupo](build/images/01-flux.pdf)

# Criptografía

La solución propuesta implementa cifrado punto a punto usando el cifrado
simétrico \Gls{AES}-256-\Gls{GCM}, con un intercambio de claves usando
\Gls{ECDH} con el algoritmo X25519 \cite{rfc7748} para la generación de claves
efímeras.

Adicionalmente, los mensajes entre los clientes están firmados con su clave de
identidad, que es una clave [ed25519](https://ed25519.cr.yp.to/).

Nótese que esta firma es innecesaria para la seguridad de la transmisión, ya
que que el mensaje está intacto también es garantizado por el algoritmo de
cifrado. Sin embargo, garantiza que quien lo envía es quien dice ser, en caso
de que hubiera un \Gls{MITM} durante el intercambio de claves.

A pesar de que en el prototipo las claves de identidad son efímeras por
conveniencia (no se ha implementado persistencia ni una base de datos de
identidades conocidas), la idea es que estas claves ed25519 pudieran ser
persistentes.

El cifrado (una vez se han acordado las claves correspondientes) y la firma de
los mensajes son relativamente sencillos, por lo que no se indagará mucho más
en profundidad en ellos en esta sección. Viven en `src/protocol/encryption.rs`
y `src/protocol/signature.rs`, respectivamente.

La clave de cifrado es efímera para cada sesión de comunicación entre dos
clientes, y está generada tras el proceso de intercambio de claves descrito en
\cref{subsec:keyexchange}.

La clave de firma se especifica durante la creación de la sesión y su parte
pública se envía como parte del mensaje de asociación a cualquiera de los
clientes.

## Intercambio de claves
\label{subsec:keyexchange}

El código que encapsula el intercambio de claves vive en
`src/protocol/key_exchange.rs`. El sistema de tipos de Rust garantiza que no
podamos usar una clave efímera (`EphemeralPrivateKey`) para más de una
operación de intercambio.

```rust
#[derive(Debug)]
enum KeyExchangeState {
    InProgress(PrivateKey),
    Completed(Arc<super::encryption::Keys>),
    Errored,
}

#[derive(Debug)]
pub struct KeyExchange {
    public_key: PublicKey,
    state: State,
}
```

Cuando se descubre un cliente nuevo, se crea un objeto `KeyExchange` en el
estado `InProgress`, con una clave privada:

```rust
impl KeyExchange {
    pub fn new() -> Result<Self, Unspecified> {
        let private = ring::agreement::EphemeralPrivateKey::generate(
            &X25519,
            &ring::rand::SystemRandom::new(),
        )?;
        let public_key = private.compute_public_key()?;
        Ok(Self {
            public_key,
            state: State::InProgress(private),
        })
    }
}
```

La clave pública se exporta en el mensaje de asociación:

```rust
impl KeyExchange {
    pub fn export_public_key(&self) -> MaybeInvalidPublicKey {
        MaybeInvalidPublicKey(self.public_key.as_ref().try_into().unwrap())
    }
}
```

Cuando recibimos la clave del otro cliente, se reemplaza el estado `InProgress`
con `Errored` (en caso de algún error criptográfico), o `Completed`, con la
clave AES256 usada para cifrar y descifrar mensajes:

```rust
impl KeyExchange {
    pub fn finish(&mut self, peer_key: &MaybeInvalidPublicKey) -> GenericResult<()> {
        if !matches!(self.state, KeyExchangeState::InProgress(..)) {
            return Err(trivial_error!("Exchange already completed"));
        }
        let result = std::mem::replace(&mut self.state, State::Errored);
        self.state = match result {
            KeyExchangeState::InProgress(private) => {
                let peer_key = UnparsedPublicKey::new(&X25519, &peer_key.0[..]);
                State::Completed(Arc::new(Keys::from_shared_secret(private, peer_key)?))
            }
            _ => unreachable!(),
        };
        Ok(())
    }
}
```

# Formato y envío de mensajes

El formato de mensaje es común para tanto el canal de control como el de
comunicación.

Consiste en una cabecera de 64 bits con un número mágico de 16 bits (`0xdead`),
un número de version de 16 bits (del cual se podrían usar 8 para flags varias,
ya que la versión actual sería siempre `1`), y la longitud del mensaje binario
en 32 bits.

Los mensajes del canal de control no van cifrados ni firmados, mientras que los
mensajes entre clientes sí. Las rutinas de envío de mensajes se encargan de
cifrar/descifrar y firmar/validar el mensaje de forma transparente.

Las rutinas de envío y recepción de mensajes son totalmente ajenas al formato
que usen los clientes. De hecho, la demo de Linux intercambia cadenas de
caracteres planas, y la demo de Android intercambia JSON, por ejemplo.

Para el canal de control se ha utilizado
[bincode](https://crates.io/crates/bincode) para codificar los mensajes de
control, ya que junto a las capacidades de meta-programación de Rust permite
derivar la codificación de los mensajes:

```rust
#[derive(Encode, Decode, Debug)]
pub enum ControlMessage {
    // ...
}

// ...

#[derive(Encode, Decode, Debug, Clone)]
pub struct DecodableMacAddr {
    is_v8: bool,
    bytes: [u8; 8],
}
```

Esto facilita los cambios en la fase de prototipado, y evita errores
innecesarios que inevitablemente pasan de otra forma.
