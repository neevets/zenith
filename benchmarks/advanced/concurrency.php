<?php
function worker($task_id) {
    Fiber::suspend($task_id);
    return $task_id * 2;
}

function storm($count) {
    $fibers = array_fill(0, $count, null);
    for ($i = 0; $i < $count; $i++) {
        $fibers[$i] = new Fiber('worker');
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
$res = storm(1000);
$end = microtime(true);
echo "PHP Fiber Storm(1k) Result: $res\n";
echo "PHP Fiber Storm(1k) Time: " . ($end - $start) . "s\n";
