<?php
function multiply($size) {
    $a = array_fill(0, $size, array_fill(0, $size, 1));
    $b = array_fill(0, $size, array_fill(0, $size, 2));
    $c = array_fill(0, $size, array_fill(0, $size, 0));

    for ($i = 0; $i < $size; $i++) {
        for ($j = 0; $j < $size; $j++) {
            $sum = 0;
            for ($k = 0; $k < $size; $k++) {
                $sum += $a[$i][$k] * $b[$k][$j];
            }
            $c[$i][$j] = $sum;
        }
    }
    return $c[0][0];
}
$start = microtime(true);
$res = multiply(100);
$end = microtime(true);
echo "PHP Matrix(100) Result: $res\n";
echo "PHP Matrix(100) Time: " . ($end - $start) . "s\n";
