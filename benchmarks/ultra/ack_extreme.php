<?php
ini_set('xdebug.max_nesting_level', '5000'); // Ack(3,9) is deep
function ack($m, $n) {
    if ($m == 0) return $n + 1;
    if ($n == 0) return ack($m - 1, 1);
    return ack($m - 1, ack($m, $n - 1));
}
$start = microtime(true);
$res = ack(3, 9);
$end = microtime(true);
echo "PHP Ackermann(3, 9) Result: $res\n";
echo "PHP Ackermann(3, 9) Time: " . ($end - $start) . "s\n";
