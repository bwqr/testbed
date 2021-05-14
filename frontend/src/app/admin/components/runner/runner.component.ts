import {AfterViewInit, Component, ElementRef, OnDestroy, OnInit, ViewChild} from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {ExperimentViewModelService} from '../../../experiment/services/experiment-view-model.service';
import {ActivatedRoute} from '@angular/router';
import {switchMap} from 'rxjs/operators';
import {SlimRunner} from '../../../experiment/models';
import {AdminViewModelService} from '../../services/admin-view-model.service';
import {interval, Subscription} from 'rxjs';
import {formats} from '../../../../defs';
import {line} from 'd3-shape/src/index.js';

@Component({
  selector: 'app-receiver-values',
  templateUrl: './runner.component.html',
  styleUrls: ['./runner.component.scss']
})
export class RunnerComponent extends MainComponent implements OnInit, OnDestroy, AfterViewInit {
  private static MAX_BUFFER_LENGTH = 10;

  runner: SlimRunner;

  receiverValuesSub: Subscription;

  receivers: number[][] = [];

  formats = formats;

  @ViewChild('canvasElement') canvasElement: ElementRef;
  @ViewChild('svgElement') svgElement: ElementRef;

  get isPageReady(): boolean {
    return !!this.runner;
  }


  constructor(
    private viewModel: AdminViewModelService,
    private experimentViewModel: ExperimentViewModelService,
    private activatedRoute: ActivatedRoute
  ) {
    super();
  }

  ngOnInit(): void {
    this.subs.add(
      this.activatedRoute.params.pipe(
        switchMap((params) => this.experimentViewModel.runner(params.id))
      )
        .subscribe(runner => {
          this.runner = runner;

          if (this.receiverValuesSub) {
            this.receiverValuesSub.unsubscribe();
          }

          this.receiverValuesSub = interval(5000).pipe(
            switchMap(_ => this.viewModel.runnerReceiversValues(runner.id))
          )
            .subscribe(res => {
              if (!res.values) {
                return;
              }

              while (res.values.length > this.receivers.length) {
                this.receivers.push([]);
              }

              for (let i = 0; i < res.values.length; i++) {
                if (this.receivers[i].length > RunnerComponent.MAX_BUFFER_LENGTH) {
                  this.receivers[i].shift();
                }
                this.receivers[i].push(res.values[i] - 50);
              }
            });
        })
    );
  }

  ngOnDestroy(): void {
    super.ngOnDestroy();

    if (this.receiverValuesSub) {
      this.receiverValuesSub.unsubscribe();
    }
  }

  ngAfterViewInit(): void {
    const svg = this.svgElement.nativeElement;
    const ctx = this.canvasElement.nativeElement.getContext('2d');

    const svgLine = line();

    svg.setAttribute('d', svgLine([
      [0, 80],
      [100, 100],
      [300, 50],
      [400, 40],
      [500, 80]
    ]));

    const d3Line = line()
      .context(ctx);

    ctx.strokeStyle = '#999';
    ctx.beginPath();
    d3Line([
      [0, 80],
      [100, 100],
      [300, 50],
      [400, 40],
      [500, 80]
    ]);
    ctx.stroke();
  }
}
