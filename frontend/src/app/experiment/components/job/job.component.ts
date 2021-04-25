import {Component, ElementRef, OnInit, ViewChild} from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {Job, SlimRunner} from '../../models';
import {ExperimentViewModelService} from '../../services/experiment-view-model.service';
import {ActivatedRoute} from '@angular/router';
import {switchMap} from 'rxjs/operators';
import {basicSetup, EditorState, EditorView} from '@codemirror/basic-setup';
import {python} from '@codemirror/lang-python';
import {formats} from '../../../../defs';


@Component({
  selector: 'app-job',
  templateUrl: './job.component.html',
  styleUrls: ['./job.component.scss']
})
export class JobComponent extends MainComponent implements OnInit {

  job: Job;
  runner: SlimRunner;
  renderedOutput: string;

  @ViewChild('code') code: ElementRef;

  editor: EditorView;

  formats = formats;

  get isPageReady(): boolean {
    return !!this.job && !!this.runner;
  }

  constructor(
    private viewModel: ExperimentViewModelService,
    private activatedRoute: ActivatedRoute
  ) {
    super();
  }

  ngOnInit(): void {
    this.subs.add(
      this.activatedRoute.params.pipe(
        switchMap(params => this.viewModel.job(params.id))
      ).subscribe(([job, runner]) => {
        this.job = job;
        this.runner = runner;

        // experiment.code is html encoded, we need to decode it
        const el = document.createElement('textarea');
        el.innerHTML = this.job.code;
        const renderedCode = el.textContent;
        el.innerHTML = this.job.output;
        this.renderedOutput = el.textContent;

        // experiment.code is html encoded, we need to decode it
        this.editor = new EditorView({
          state: EditorState.create({
            doc: renderedCode,
            extensions: [
              basicSetup,
              python(),
              EditorState.tabSize.of(4),
              EditorView.editable.of(false)
            ]
          }),
          parent: this.code.nativeElement
        });
      })
    );
  }

}
