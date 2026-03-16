<?php
function task($id) {
    Fiber::suspend($id);
    return $id * 2;
}
function storm($count) {
    $fibers = array_fill(0, $count, null);
    for ($i = 0; $i < $count; $i++) {
        $fibers[$i] = new Fiber('task');
        $fibers[$i]->start($i);
    }
    $sum = 0;
    for ($j = 0; $j < $count; $j++) {
        $f = $fibers[$j];
        $f->resume();
        $sum += $f->getReturn();
    }
    return $sum;
}
$start = microtime(true);
$res = storm(10000);
$end = microtime(true);
echo "PHP Fibers(10k) Result: $res\n";
echo "PHP Fibers(10k) Time: " . ($end - $start) . "s\n";
