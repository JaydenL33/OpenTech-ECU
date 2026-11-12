"""Required route-3 pre-routes, in millimetres and P1–P13 table order."""

PREROUTES = (
    dict(id='P1', net='GND', layer='F.Cu', start=(157.325, 117.5),
         end=(159.2, 117.5), width=0.2),
    dict(id='P2', net='GND', layer='via', position=(159.2, 117.5),
         size=0.6, drill=0.3),
    dict(id='P3', net='GND', layer='F.Cu', start=(159.0, 126.675),
         end=(159.0, 124.8), width=0.2),
    dict(id='P4', net='GND', layer='via', position=(159.0, 124.8),
         size=0.6, drill=0.3),
    dict(id='P5', net='GND', layer='F.Cu', start=(170.5, 126.675),
         end=(170.5, 124.8), width=0.2),
    dict(id='P6', net='GND', layer='via', position=(170.5, 124.8),
         size=0.6, drill=0.3),
    dict(id='P7', net='GND', layer='F.Cu', start=(172.675, 113.5),
         end=(170.8, 113.5), width=0.2),
    dict(id='P8', net='GND', layer='via', position=(170.8, 113.5),
         size=0.6, drill=0.3),
    dict(id='P9', net='GND', layer='F.Cu', start=(159.5, 111.325),
         end=(159.5, 113.2), width=0.2),
    dict(id='P10', net='GND', layer='via', position=(159.5, 113.2),
         size=0.6, drill=0.3),
    dict(id='P11', net='GND', layer='F.Cu', start=(23.7, 89.225),
         end=(25.175, 89.225), width=0.2),
    dict(id='P12', net='INJ4_OUT', layer='B.Cu', start=(150.0, 67.385),
         end=(156.8, 63.175), width=0.2),
    dict(id='P13', net='GND', layer='F.Cu', start=(172.385, 149.925),
         end=(174.585, 151.625), width=0.2),
)

SERVED_PADS = (
    ('U200', '10'), ('U200', '26'), ('U200', '49'),
    ('U200', '74'), ('U200', '99'), ('U104', '3'),
    ('Y200', '4'),
)
