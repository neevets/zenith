<?php
function ack($m, $n) {
    if ($m == 0) return $n + 1;
    if ($n == 0) return ack($m - 1, 1);
    return ack($m - 1, ack($m, $n - 1));
}
$start = microtime(true);
$res = ack(3, 7);
$end = microtime(true);
echo "PHP Ackermann(3, 7) Result: $res\n";
echo "PHP Ackermann(3, 7) Time: " . ($end - $start) . "s\n";
