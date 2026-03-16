# Referencia de Sintaxis de Zenith

Zenith es un lenguaje moderno, tipado y seguro que transpila a PHP 8.1+. Esta página contiene la guía completa de su sintaxis.

## Comentarios
```zenith
// Comentario de una sola línea
/* Comentario 
   multilínea */
```

## Variables y Tipos
Las variables siempre comienzan con `$` (siguiendo la herencia de PHP) y se declaran con `let`.

```zenith
let $nombre = "Zenith";            // string (inferido)
let $version: float = 1.0;         // float (explícito)
let $activado: bool = true;        // bool
let $contador: int = 42;           // int
let $lista = [1, 2, 3];            // array
```

## Cadenas de Texto e Interpolación
Zenith usa llaves `{}` para interpolar variables directamente en cadenas con comillas dobles.

```zenith
let $usuario = "Alice";
println("Hola, { $usuario }!"); // Imprime: Hola, Alice!
```

## Operador Pipe (`|>`)
Permite encadenar funciones de forma legible, pasando el resultado de la izquierda como primer argumento a la función de la derecha.

```zenith
"  hola mundo  " 
    |> trim() 
    |> strtoupper() 
    |> println(); // Imprime: HOLA MUNDO
```

## Estructuras de Control

### If / Else
```zenith
if ($puntos > 10) {
    println("¡Ganaste!");
} else {
    println("Sigue intentando");
}
```

### Bucle For
```zenith
for ($item in $lista) {
    println("Elemento: { $item }");
}
```

### Expresión Match
Es una versión más potente y expresiva que `switch`. Devuelve un valor.

```zenith
let $resultado = match($codigo) {
    200 => "OK",
    404 => "No encontrado",
    500 => "Error de servidor",
    default => "Código desconocido"
};
```

## Funciones
Se definen con la palabra clave `fn`.

```zenith
// Función flecha (arrow function)
let $doble = fn($n: int): int => $n * 2;

// Uso
println($doble(5)); // 10
```

## Concurrencia (Fibers)
Zenith facilita el uso de Fibers de PHP mediante bloques `spawn`.

```zenith
let $proceso = spawn {
    yield "Paso 1";
    yield "Paso 2";
};

println($proceso.resume()); // Imprime el valor de yield
```

## Ruteo Nativo (Web-First)
Zenith tiene ruteo integrado en la sintaxis.

```zenith
route GET "/perfil/{$id}" => {
    println("Cargando perfil {$id}");
}
```

## Bloques de Consulta SQL (First-class SQL)
El SQL no es un string, es parte del lenguaje.

```zenith
// Conexión nativa
db.connect("mysql:host=localhost;dbname=test", "root", "");

// Consulta segura
let $usuarios = query {
    SELECT name FROM users WHERE id == 1
};
```

## Tubería de Sanitización (`!>`)
Operador dedicado a la seguridad web para limpiar datos antes de imprimirlos.

```zenith
let $bio = "<script>alert(1)</script> Hola Zenith!";
println($bio !> "html"); // Escapa automáticamente el contenido
```

## Atributos de Comportamiento (`#[...]`)
Decoradores que gestionan el estado y comportamiento de forma declarativa.

```zenith
#[Session("user_id")]
let $uid = 0; // Se inicializa con el valor de la sesión si existe
```

## Sistema de Archivos
Acceso seguro mediante permisos.

```zenith
file.write("notas.txt", "Contenido importante");
let $texto = file.read("notas.txt");
```

## Pruebas (Testing)
El soporte para tests es nativo en el lenguaje.

```zenith
test "verificar suma" {
    let $suma = 2 + 2;
    z_assert($suma == 4, "La suma debe ser 4");
}
```
