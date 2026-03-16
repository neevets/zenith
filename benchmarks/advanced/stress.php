<?php
class Node {
    public $id;
    public $val;
    public function __construct($id, $val) {
        $this->id = $id;
        $this->val = $val;
    }
}
function stress($count) {
    $list = array_fill(0, $count, null);
    for ($i = 0; $i < $count; $i++) {
        $list[$i] = new Node($i, $i * 2);
    }
    return count($list);
}
$start = microtime(true);
$res = stress(100000);
$end = microtime(true);
echo "PHP Object Stress(100k) Result: $res\n";
echo "PHP Object Stress(100k) Time: " . ($end - $start) . "s\n";
